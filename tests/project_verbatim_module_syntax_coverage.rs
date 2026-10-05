use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn project_directory() -> PathBuf {
    std::env::temp_dir().join(format!(
        "tsrzl-verbatim-module-syntax-{}",
        std::process::id()
    ))
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("the project directory can be created");
    }
    fs::write(path, contents).expect("the project fixture can be written");
}

fn run_project(directory: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .current_dir(directory)
        .args(["--project", "."])
        .output()
        .expect("the compiler CLI can be started")
}

// Pinned TypeScript 7.0.2 fixture: compiler/isolatedModulesShadowGlobalTypeNotValue.ts.
// The ESM project isolates TS1484 from TS1295 in the fixture's CommonJS variant.
#[test]
fn should_require_type_only_import_given_verbatim_module_syntax_when_compiling_project() {
    // Arrange
    let directory = project_directory();
    write_file(
        &directory.join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"es2022","target":"es2015","verbatimModuleSyntax":true,"noEmit":true},"files":["types.ts","main.ts"]}"#,
    );
    write_file(
        &directory.join("types.ts"),
        "export interface Shape { width: number; }\n",
    );
    write_file(
        &directory.join("main.ts"),
        "import { Shape } from './types';\nconst shape: Shape = { width: 1 };\n",
    );

    // Act
    let output = run_project(&directory);
    let diagnostics = String::from_utf8_lossy(&output.stderr);

    // Assert
    assert!(
        diagnostics.contains("TS1484"),
        "verbatimModuleSyntax should require an import type declaration, got {diagnostics}"
    );
}
