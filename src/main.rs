use std::collections::{HashSet, VecDeque};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

mod jsonc;
mod project_config;

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::module_resolver;
use tsrzl::source_file::SourceFile;
use tsrzl::syntax::{Statement, SyntaxTree};

fn main() -> ExitCode {
    match run(env::args_os().skip(1)) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("tsrzl: {message}");
            ExitCode::FAILURE
        }
    }
}

struct CliArguments {
    target: Option<ScriptTarget>,
    module: Option<ModuleKind>,
    strict_null_checks: Option<bool>,
    declaration: bool,
    emit_declaration_only: bool,
    output_directory: Option<PathBuf>,
    project_path: Option<PathBuf>,
    input_paths: Vec<PathBuf>,
}

fn run(arguments: impl IntoIterator<Item = OsString>) -> Result<ExitCode, String> {
    let Some(arguments) = parse_arguments(arguments)? else {
        return Ok(ExitCode::SUCCESS);
    };
    compile_arguments(arguments)
}

fn parse_arguments(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<Option<CliArguments>, String> {
    let mut arguments = arguments.into_iter().peekable();
    let mut target = None;
    let mut module = None;
    let mut strict_null_checks = None;
    let mut declaration = false;
    let mut emit_declaration_only = false;
    let mut output_directory = None;
    let mut project_path = None;
    let mut input_paths = Vec::new();

    while let Some(argument) = arguments.next() {
        match argument.to_str() {
            Some("--help" | "-h") => {
                print_help();
                return Ok(None);
            }
            Some("--declaration") => declaration = true,
            Some("--emitDeclarationOnly") => emit_declaration_only = true,
            Some("--strictNullChecks") => {
                strict_null_checks =
                    Some(match arguments.peek().and_then(|value| value.to_str()) {
                        Some("true") => {
                            arguments.next();
                            true
                        }
                        Some("false") => {
                            arguments.next();
                            false
                        }
                        _ => true,
                    });
            }
            Some("--strictNullChecks=true") => strict_null_checks = Some(true),
            Some("--strictNullChecks=false") => strict_null_checks = Some(false),
            Some("--project" | "-p") => {
                project_path = Some(PathBuf::from(
                    arguments
                        .next()
                        .ok_or_else(|| "missing path after --project".to_owned())?,
                ));
            }
            Some("--target") => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "missing value after --target".to_owned())?;
                let value = value
                    .to_str()
                    .ok_or_else(|| "the target name must be valid UTF-8".to_owned())?;
                target = Some(parse_target(value)?);
            }
            Some("--module") => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "missing value after --module".to_owned())?;
                let value = value
                    .to_str()
                    .ok_or_else(|| "the module name must be valid UTF-8".to_owned())?;
                module = Some(parse_module(value)?);
            }
            Some("--out-dir") => {
                output_directory =
                    Some(PathBuf::from(arguments.next().ok_or_else(|| {
                        "missing directory after --out-dir".to_owned()
                    })?));
            }
            Some(argument) if argument.starts_with('-') => {
                return Err(format!("unknown option: {argument}"));
            }
            _ => input_paths.push(PathBuf::from(argument)),
        }
    }

    Ok(Some(CliArguments {
        target,
        module,
        strict_null_checks,
        declaration,
        emit_declaration_only,
        output_directory,
        project_path,
        input_paths,
    }))
}

fn compile_arguments(arguments: CliArguments) -> Result<ExitCode, String> {
    let CliArguments {
        target,
        module,
        strict_null_checks,
        declaration,
        emit_declaration_only,
        output_directory,
        project_path,
        input_paths,
    } = arguments;
    if project_path.is_some() && !input_paths.is_empty() {
        return Err("--project cannot be combined with source file arguments".to_owned());
    }
    if input_paths.is_empty() && project_path.is_none() {
        return Err("provide one or more TypeScript source paths (try --help)".to_owned());
    }
    let project_configuration = project_path
        .as_deref()
        .map(project_config::load)
        .transpose()?;
    let declaration = declaration
        || project_configuration
            .as_ref()
            .and_then(|configuration| configuration.declaration)
            .unwrap_or(false);
    let emit_declaration_only = emit_declaration_only
        || project_configuration
            .as_ref()
            .and_then(|configuration| configuration.emit_declaration_only)
            .unwrap_or(false);
    if emit_declaration_only && !declaration {
        eprintln!(
            "error TS5069: Option 'emitDeclarationOnly' cannot be specified without specifying option 'declaration' or option 'composite'."
        );
        return Ok(ExitCode::from(2));
    }
    let root_paths = project_configuration
        .as_ref()
        .map_or(input_paths, |configuration| {
            configuration.root_files.clone()
        });
    if root_paths.is_empty() {
        return Err("the project configuration did not select any source files".to_owned());
    }
    let source_files = load_source_files(&root_paths)?;
    let configured_target = project_configuration
        .as_ref()
        .and_then(|configuration| configuration.target.as_deref());
    if configured_target.is_some_and(|value| value.eq_ignore_ascii_case("ES5")) {
        eprintln!(
            "error TS5108: Option 'target=ES5' has been removed. Please remove it from your configuration."
        );
        return Ok(ExitCode::from(2));
    }
    let configured_target = configured_target.map(parse_target).transpose()?;
    let target = target.or(configured_target).unwrap_or(ScriptTarget::Es2025);
    let mut options = CompilerOptions::new(target);
    let configured_strict_null_checks = project_configuration
        .as_ref()
        .and_then(|configuration| configuration.strict_null_checks);
    let strict_null_checks = strict_null_checks
        .or(configured_strict_null_checks)
        .unwrap_or(true);
    let configured_module = project_configuration
        .as_ref()
        .and_then(|configuration| configuration.module.as_deref())
        .map(parse_module)
        .transpose()?;
    if let Some(module) = module.or(configured_module) {
        options = options.with_module(module);
    }
    let configured_output_directory = project_configuration
        .as_ref()
        .and_then(|configuration| configuration.out_dir.clone());
    let output_directory = output_directory.or(configured_output_directory);
    options = options
        .with_declaration(declaration)
        .with_emit_declaration_only(emit_declaration_only)
        .with_strict_null_checks(strict_null_checks);
    let compiler = Compiler::with_options(options);
    let result = compiler.compile_sources(source_files);

    write_outputs(result.emitted_files(), output_directory.as_deref())?;
    for diagnostic in result.diagnostics() {
        eprintln!("TS{}: {}", diagnostic.code(), diagnostic.message());
    }

    if result.diagnostics().is_empty() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}

fn read_source_file(path: &Path) -> Result<SourceFile, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    SourceFile::from_path(path, text).map_err(|error| format!("{}: {error}", path.display()))
}

fn load_source_files(root_paths: &[PathBuf]) -> Result<Vec<SourceFile>, String> {
    let mut pending_paths = root_paths.iter().cloned().collect::<VecDeque<_>>();
    let mut loaded_paths = HashSet::new();
    let mut source_files = Vec::new();

    while let Some(path) = pending_paths.pop_front() {
        let source_file = read_source_file(&path)?;
        let identity = fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        if !loaded_paths.insert(identity) {
            continue;
        }

        let syntax_tree = SyntaxTree::parse(source_file.clone());
        for statement in syntax_tree.program().statements() {
            let module_specifier = match statement.declaration() {
                Statement::ImportDeclaration(import) => import.module_specifier(),
                Statement::ExportAll(export) => export.module_specifier(),
                Statement::ExportNamedFrom(export) => export.module_specifier(),
                _ => continue,
            };
            if let Some(imported_path) = module_resolver::resolve_from_disk(&path, module_specifier)
            {
                pending_paths.push_back(imported_path);
            }
        }
        source_files.push(source_file);
    }

    Ok(source_files)
}

fn write_outputs(
    emitted_files: &[tsrzl::compiler::EmittedFile],
    output_directory: Option<&Path>,
) -> Result<(), String> {
    for emitted_file in emitted_files {
        let output_path = match output_directory {
            Some(directory) => {
                let file_name = emitted_file.path().file_name().ok_or_else(|| {
                    format!(
                        "output path has no file name: {}",
                        emitted_file.path().display()
                    )
                })?;
                directory.join(file_name)
            }
            None => emitted_file.path().to_path_buf(),
        };

        if let Some(parent) = output_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("could not create {}: {error}", parent.display()))?;
            }
        }
        fs::write(&output_path, emitted_file.text())
            .map_err(|error| format!("could not write {}: {error}", output_path.display()))?;
    }
    Ok(())
}

fn parse_target(value: &str) -> Result<ScriptTarget, String> {
    match value.to_ascii_lowercase().as_str() {
        "es5" => Err("target es5 has been removed in TypeScript 7.0".to_owned()),
        "es2015" | "es6" => Ok(ScriptTarget::Es2015),
        "es2016" => Ok(ScriptTarget::Es2016),
        "es2017" => Ok(ScriptTarget::Es2017),
        "es2018" => Ok(ScriptTarget::Es2018),
        "es2019" => Ok(ScriptTarget::Es2019),
        "es2020" => Ok(ScriptTarget::Es2020),
        "es2021" => Ok(ScriptTarget::Es2021),
        "es2022" => Ok(ScriptTarget::Es2022),
        "es2023" => Ok(ScriptTarget::Es2023),
        "es2024" => Ok(ScriptTarget::Es2024),
        "es2025" => Ok(ScriptTarget::Es2025),
        "esnext" | "latest" => Ok(ScriptTarget::Latest),
        _ => Err(format!("unsupported target: {value}")),
    }
}

fn parse_module(value: &str) -> Result<ModuleKind, String> {
    match value.to_ascii_lowercase().as_str() {
        "commonjs" => Ok(ModuleKind::CommonJs),
        "es2015" | "es6" => Ok(ModuleKind::Es2015),
        "es2020" => Ok(ModuleKind::Es2020),
        "es2022" => Ok(ModuleKind::Es2022),
        "esnext" => Ok(ModuleKind::EsNext),
        "preserve" => Ok(ModuleKind::Preserve),
        _ => Err(format!("unsupported module: {value}")),
    }
}

fn print_help() {
    println!(
        "Usage: tsrzl [--project FILE|DIRECTORY] [--target es2015|es2016|es2017|es2018|es2019|es2020|es2021|es2022|es2023|es2024|es2025|latest] [--module commonjs|es2015|es2020|es2022|esnext|preserve] [--strictNullChecks [true|false]] [--declaration] [--emitDeclarationOnly] [--out-dir DIRECTORY] [FILE...]\n\nCompile TypeScript source files and write JavaScript and optional declaration output."
    );
}
