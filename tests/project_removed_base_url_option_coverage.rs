use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrzl-removed-base-url-option-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("the project directory can be created");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn write(&self, relative_path: &str, contents: &str) {
        fs::write(self.root.join(relative_path), contents)
            .expect("the project fixture can be written");
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
        let _ = fs::remove_dir_all(&self.root);
    }
}

// Pinned TypeScript-Go source: internal/compiler/program.go removed-option diagnostics.
// Direct TypeScript-Go 7.0.2 reports TS5102 for baseUrl in a project configuration.
#[test]
fn should_report_removed_base_url_given_project_configuration_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"files":["main.ts"],"compilerOptions":{"baseUrl":"."}}"#,
    );
    project.write("main.ts", "export const answer = 42;");

    // Act
    let output = project.run_cli();

    // Assert
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        diagnostics.contains("TS5102") && diagnostics.contains("paths"),
        "expected TS5102 and its paths replacement suggestion for removed baseUrl, got {diagnostics:?}"
    );
}
