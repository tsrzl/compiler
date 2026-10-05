use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn project_directory() -> PathBuf {
    std::env::temp_dir().join(format!("tsrzl-transitive-reference-{}", std::process::id()))
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("the project directory can be created");
    }
    fs::write(path, contents).expect("the project source can be written");
}

// Pinned TypeScript 7.0.2 project case: projects/reference-1/main.ts and lib/classB.ts.
#[test]
fn should_resolve_transitive_triple_slash_reference_given_nested_reference_when_compiling_project()
{
    // Arrange
    let directory = project_directory();
    write_file(
        &directory.join("tsconfig.json"),
        r#"{"files":["main.ts"],"compilerOptions":{"strict":false}}"#,
    );
    write_file(
        &directory.join("main.ts"),
        "/// <reference path=\"./lib/intermediate.d.ts\" />\nlet value: TransitiveType;",
    );
    write_file(
        &directory.join("lib/intermediate.d.ts"),
        "/// <reference path=\"./base.d.ts\" />\n",
    );
    write_file(
        &directory.join("lib/base.d.ts"),
        "declare class TransitiveType {}\n",
    );

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .current_dir(&directory)
        .args(["--project", "."])
        .output()
        .expect("the compiler CLI can be started");
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = fs::remove_dir_all(&directory);

    // Assert
    assert!(output.status.success(), "{diagnostics}");
}
