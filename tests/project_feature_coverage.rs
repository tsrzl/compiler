use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

fn project_directory(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "tsrzl-project-feature-{}-{name}",
        std::process::id()
    ))
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("the source directory can be created");
    }
    fs::write(path, contents).expect("the fixture file can be written");
}

fn run_cli(arguments: &[&std::ffi::OsStr], working_directory: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .current_dir(working_directory)
        .args(arguments)
        .output()
        .expect("the compiler CLI can be started")
}

// Pinned TypeScript input: compiler/resolutionCandidateFromPackageJsonField1.ts.
#[test]
fn should_resolve_path_mapped_import_given_paths_project_option_when_running_compiler_cli() {
    // Arrange
    let directory = project_directory("paths");
    write_file(
        &directory.join("tsconfig.json"),
        r#"{"compilerOptions":{"paths":{"@lib/*":["./src/*"]}},"files":["main.ts","src/answer.ts"]}"#,
    );
    write_file(
        &directory.join("main.ts"),
        "import { answer } from '@lib/answer'; const result: number = answer;",
    );
    write_file(
        &directory.join("src/answer.ts"),
        "export const answer: number = 42;",
    );

    // Act
    let process = run_cli(&["--project".as_ref(), directory.as_os_str()], &directory);

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
}

// Pinned TypeScript input: conformance/moduleResolution/packageJsonMain.ts.
#[test]
fn should_resolve_package_main_given_non_index_package_entry_when_running_compiler_cli() {
    // Arrange
    let directory = project_directory("package-main");
    write_file(
        &directory.join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"commonjs","strict":false},"files":["main.ts"]}"#,
    );
    write_file(
        &directory.join("main.ts"),
        "import { answer } from 'answer-package'; const result: number = answer;",
    );
    write_file(
        &directory.join("node_modules/answer-package/package.json"),
        r#"{"main":"src/entry"}"#,
    );
    write_file(
        &directory.join("node_modules/answer-package/src/entry.js"),
        "exports.answer = 42;",
    );

    // Act
    let process = run_cli(&["--project".as_ref(), directory.as_os_str()], &directory);

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
}

#[test]
fn should_report_removed_classic_module_resolution_given_project_option_when_running_compiler_cli()
{
    // Pinned project case: projects/RelativePaths/app.ts.
    // TS-Go 7.0.2 reports TS5108 because moduleResolution=Classic was removed.
    // Arrange
    let directory = project_directory("relative-paths-classic-resolution");
    write_file(
        &directory.join("tsconfig.json"),
        r#"{"files":["app.ts"],"compilerOptions":{"moduleResolution":"classic"}}"#,
    );
    write_file(
        &directory.join("app.ts"),
        "import a = require('A/a');\na.A();",
    );

    // Act
    let process = run_cli(&["--project".as_ref(), directory.as_os_str()], &directory);

    // Assert
    assert_eq!(
        String::from_utf8_lossy(&process.stderr),
        "error TS5108: Option 'moduleResolution=Classic' has been removed. Please remove it from your configuration.\n"
    );
}

// Pinned TypeScript input: conformance/moduleResolution/nodeModulesAtTypesPriority.ts.
#[test]
fn should_resolve_at_types_package_given_bare_import_when_compiling_sources() {
    // Arrange
    let importing_source = SourceFile::from_path(
        Path::new("packages/a/index.ts"),
        "import { answer } from 'answer-package'; const label: string = answer;",
    )
    .expect("the importing TypeScript path has a supported source kind");
    let type_source = SourceFile::from_path(
        Path::new("node_modules/@types/answer-package/index.d.ts"),
        "export declare const answer: number;",
    )
    .expect("the package declaration path has a supported source kind");

    // Act
    let result = Compiler::new().compile_sources([importing_source, type_source]);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .all(|diagnostic| diagnostic.code() != 2307),
        "the package should resolve through its @types declaration"
    );
}

// Pinned TypeScript input: conformance/node/nodeModulesPackageExports.ts.
#[test]
fn should_resolve_exported_package_subpath_given_node16_project_when_running_compiler_cli() {
    // Arrange
    let directory = project_directory("package-exports");
    write_file(
        &directory.join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"node16","target":"es2022","strict":true,"outDir":"dist"},"files":["main.mts"]}"#,
    );
    write_file(
        &directory.join("main.mts"),
        "import { answer } from 'answer-package/answer'; export const result: number = answer;",
    );
    write_file(
        &directory.join("node_modules/answer-package/package.json"),
        r#"{"name":"answer-package","type":"module","exports":{"./answer":{"types":"./types/answer.d.ts","default":"./dist/answer.js"}}}"#,
    );
    write_file(
        &directory.join("node_modules/answer-package/types/answer.d.ts"),
        "export declare const answer: number;",
    );
    write_file(
        &directory.join("node_modules/answer-package/dist/answer.js"),
        "export const answer = 42;",
    );

    // Act
    let process = run_cli(&["--project".as_ref(), directory.as_os_str()], &directory);

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
}

// Pinned TypeScript-Go baseline: tsc/noEmit/when-project-has-strict-true.js.
#[test]
fn should_suppress_javascript_output_given_no_emit_option_when_running_compiler_cli() {
    // Arrange
    let directory = project_directory("no-emit");
    let input_path = directory.join("input.ts");
    write_file(&input_path, "const answer: number = 42;");

    // Act
    let process = run_cli(&["--noEmit".as_ref(), input_path.as_os_str()], &directory);

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert!(!directory.join("input.js").exists());
}

// Pinned TypeScript input: compiler/commonSourceDirectory.ts.
#[test]
fn should_preserve_source_subdirectories_given_out_dir_option_when_running_compiler_cli() {
    // Arrange
    let directory = project_directory("out-dir-layout");
    write_file(
        &directory.join("src/main.ts"),
        "import { answer } from './features/answer'; export const result = answer;",
    );
    write_file(
        &directory.join("src/features/answer.ts"),
        "export const answer: number = 42;",
    );

    // Act
    let process = run_cli(
        &[
            "--out-dir".as_ref(),
            Path::new("out").as_os_str(),
            Path::new("src/main.ts").as_os_str(),
        ],
        &directory,
    );

    // Assert
    assert!(
        process.status.success(),
        "{}",
        String::from_utf8_lossy(&process.stderr)
    );
    assert!(directory.join("out/features/answer.js").exists());
}
