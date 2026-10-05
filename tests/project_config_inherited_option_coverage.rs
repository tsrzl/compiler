use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-project-config-inherited-option-{}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("the temporary project directory can be created");
        Self { path }
    }

    fn write(&self, relative_path: &str, contents: &str) {
        fs::write(self.path.join(relative_path), contents)
            .expect("the project configuration or source can be written");
    }

    fn run_cli(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .current_dir(&self.path)
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

// Pinned TypeScript configuration fixture: compiler/configFileExtendsAsList.ts.
#[test]
fn should_report_implicit_any_given_no_implicit_any_in_parent_config_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "base.json",
        r#"{"compilerOptions":{"noImplicitAny":true,"noEmit":true}}"#,
    );
    project.write(
        "tsconfig.json",
        r#"{"extends":"./base.json","files":["main.ts"]}"#,
    );
    project.write("main.ts", "function identity(value) {}\n");

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
        "the child project should inherit noImplicitAny; got {diagnostics}"
    );
}
