use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tsrzl::compiler::{Compiler, CompilerOptions, ModuleKind, ScriptTarget};
use tsrzl::source_file::SourceFile;

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-project-transpile-{}-{name}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("the temporary project directory can be created");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, relative_path: &str, contents: &str) {
        let path = self.path.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("the fixture directory can be created");
        }
        fs::write(path, contents).expect("the fixture file can be written");
    }

    fn run_cli(&self, arguments: &[&std::ffi::OsStr]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .current_dir(self.path())
            .args(arguments)
            .output()
            .expect("the compiler CLI can be started")
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

// Pinned project fixture: projects/jsFileCompilation/DifferentNamesNotSpecifiedWithAllowJs/a.ts.
// Runner config: project/jsFileCompilationDifferentNamesNotSpecifiedWithAllowJs.json.
#[test]
fn should_reject_legacy_out_file_given_allow_js_project_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new("allow-js-out-file");
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"outFile":"test.js","allowJs":true,"declaration":true}}"#,
    );
    project.write("a.ts", "var test = 10;");
    project.write("b.js", "var test2 = 10; // Should get compiled");

    // Act
    let process = project.run_cli(&["--project".as_ref(), project.path().as_os_str()]);

    // Assert
    assert!(!process.status.success());
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );
    assert!(diagnostics.contains("TS5102"), "{diagnostics}");
}

// Pinned project fixture: projects/declarationDir/a.ts and projects/declarationDir/subfolder/b.ts.
// Runner options: project/declarationDir.json (declarationDir=declarations).
#[test]
fn should_write_declarations_to_declaration_dir_given_declaration_dir_project_option_when_running_compiler_cli()
 {
    // Arrange
    let project = TemporaryProject::new("declaration-dir");
    project.write(
        "tsconfig.json",
        r#"{"files":["a.ts","subfolder/b.ts","subfolder/c.ts"],"compilerOptions":{"rootDir":".","strict":false,"declaration":true,"declarationDir":"declarations"}}"#,
    );
    project.write(
        "a.ts",
        "import {B} from './subfolder/b';\nexport class A {\n    b: B;\n}",
    );
    project.write("subfolder/b.ts", "export class B {\n    \n}");
    project.write(
        "subfolder/c.ts",
        "import {A} from '../a';\n\nexport class C {\n    a: A;\n}",
    );

    // Act
    let process = project.run_cli(&["--project".as_ref(), project.path().as_os_str()]);

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert!(project.path().join("declarations/a.d.ts").exists());
}

// Pinned project fixture: projects/rootDirectory/FolderA/FolderB/fileB.ts.
// Runner options: project/rootDirectoryErrors.json (rootDir=FolderA/FolderB/FolderC).
#[test]
fn should_report_source_outside_root_dir_given_project_root_dir_option_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new("root-dir-error");
    project.write(
        "tsconfig.json",
        r#"{"files":["FolderA/FolderB/fileB.ts","FolderA/FolderB/FolderC/fileC.ts"],"compilerOptions":{"rootDir":"FolderA/FolderB/FolderC","outDir":"outdir/simple","declaration":true,"strict":false}}"#,
    );
    project.write(
        "FolderA/FolderB/fileB.ts",
        "/// <reference path='FolderC/fileC.ts'/>\nclass B {\n    public c: C;\n}",
    );
    project.write("FolderA/FolderB/FolderC/fileC.ts", "class C {}");

    // Act
    let process = project.run_cli(&["--project".as_ref(), project.path().as_os_str()]);

    // Assert
    assert!(!process.status.success());
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );
    assert!(diagnostics.contains("TS6059"), "{diagnostics}");
}

// Pinned transpile fixture: transpile/jsWithInlineSourceMapBasic.ts.
// The selected configuration is inlineSourceMap=true, target=es6.
#[test]
fn should_emit_inline_source_map_given_inline_source_map_option_when_compiling() {
    // Arrange
    let project = TemporaryProject::new("inline-source-map");
    project.write("input.ts", "const answer: number = 42;");

    // Act
    let process = project.run_cli(&[
        "--inlineSourceMap".as_ref(),
        "--target".as_ref(),
        "es2015".as_ref(),
        project.path().join("input.ts").as_os_str(),
    ]);

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    let javascript = fs::read_to_string(project.path().join("input.js"))
        .expect("the emitted JavaScript can be read");
    assert!(javascript.contains("sourceMappingURL=data:application/json;base64,"));
}

// Pinned transpile fixture: transpile/declarationBasicSyntax.ts.
// Selected configuration: declaration=true, declarationMap=true, target=es6.
#[test]
fn should_emit_declaration_map_given_declaration_map_option_when_compiling() {
    // Arrange
    let project = TemporaryProject::new("declaration-map");
    project.write("input.ts", "export interface Answer { value: number; }");

    // Act
    let process = project.run_cli(&[
        "--declaration".as_ref(),
        "--declarationMap".as_ref(),
        "--target".as_ref(),
        "es2015".as_ref(),
        project.path().join("input.ts").as_os_str(),
    ]);

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert!(project.path().join("input.d.ts.map").exists());
}

// Pinned transpile fixture: transpile/declarationCrossFileInferences.ts.
// Options: target=es2015, module=commonjs, declaration=true.
#[test]
fn should_infer_cross_file_declaration_type_given_transitive_factory_call_when_compiling_sources() {
    // Arrange
    let sources = [
        SourceFile::from_path(
            Path::new("defines.ts"),
            "export class A { field = { x: 1 } }",
        )
        .expect("the defining TypeScript path has a supported source kind"),
        SourceFile::from_path(
            Path::new("consumes.ts"),
            "import { A } from './defines.js'; export function create() { return new A(); }",
        )
        .expect("the consuming TypeScript path has a supported source kind"),
        SourceFile::from_path(
            Path::new("exposes.ts"),
            "import { create } from './consumes.js'; export const value = create();",
        )
        .expect("the exporting TypeScript path has a supported source kind"),
    ];
    let options = CompilerOptions::new(ScriptTarget::Es2015)
        .with_module(ModuleKind::CommonJs)
        .with_declaration(true);

    // Act
    let result = Compiler::with_options(options).compile_sources(sources);

    // Assert
    let declaration = result
        .emitted_files()
        .iter()
        .find(|file| file.path() == Path::new("exposes.d.ts"))
        .expect("the exposed declaration file is emitted");
    assert_eq!(
        declaration.text(),
        "export declare const value: import(\"./defines.js\").A;\n"
    );
}
