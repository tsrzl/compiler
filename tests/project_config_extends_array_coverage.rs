use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-project-config-extends-array-{}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("the temporary project directory can be created");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, relative_path: &str, contents: &str) {
        fs::write(self.path.join(relative_path), contents)
            .expect("the project configuration or source can be written");
    }

    fn run_cli(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .current_dir(self.path())
            .args(["--project", "."])
            .output()
            .expect("the compiler CLI can be started")
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

// Pinned fixture: compiler/configFileExtendsAsList.ts, with two inherited option files.
#[test]
fn should_report_implicit_any_given_array_extends_project_configuration_when_running_compiler_cli()
{
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig1.json",
        r#"{"compilerOptions":{"strictNullChecks":true}}"#,
    );
    project.write(
        "tsconfig2.json",
        r#"{"compilerOptions":{"noImplicitAny":true}}"#,
    );
    project.write(
        "tsconfig.json",
        r#"{"extends":["./tsconfig1.json","./tsconfig2.json"]}"#,
    );
    project.write("index.ts", "function f(value) {}\n");

    // Act
    let output = project.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(
        diagnostics.contains("TS7006"),
        "the second inherited config should enable noImplicitAny; got {diagnostics}"
    );
}
