use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("tsrzl-typings-suggestion-{}", std::process::id()));
        fs::create_dir_all(&path).expect("the temporary project directory can be created");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, relative_path: &str, contents: &str) {
        let path = self.path.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("the project fixture directory can be created");
        }
        fs::write(path, contents).expect("the project fixture can be written");
    }

    fn run_cli(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .current_dir(self.path())
            .arg("--project")
            .arg(self.path())
            .output()
            .expect("the compiler CLI can be started")
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn should_suggest_node_types_given_missing_module_global_and_empty_types_option_when_compiling_project()
 {
    // TypeScript 7.0.2 case: conformance/typings/typingsSuggestion1.ts.
    // Setting types to [] makes the oracle report TS2591 with an @types/node suggestion.
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"es2015","types":[]},"files":["a.ts"]}"#,
    );
    project.write("a.ts", "module.exports = 1;");

    // Act
    let process = project.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );

    // Assert
    assert!(
        diagnostics.contains("TS2591"),
        "expected Node.js type suggestion diagnostic, got: {diagnostics}"
    );
}
