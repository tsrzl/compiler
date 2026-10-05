use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-project-circular-return-inference-{}",
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
            .expect("the project source file can be written");
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

// Pinned project group: projects/CircularReferencing/consume.ts imports decl.ts, which imports it back.
// ESM imports keep this focused on cyclic return-type inference instead of import-equals parsing.
#[test]
fn should_report_implicit_any_return_given_circular_imported_calls_when_compiling_project() {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"files":["consume.ts"],"compilerOptions":{"target":"es2015","module":"commonjs","strict":true}}"#,
    );
    project.write(
        "consume.ts",
        "import { call as declCall } from \"./decl\";\nexport function call() { return declCall(); }\n",
    );
    project.write(
        "decl.ts",
        "import { call as consumeCall } from \"./consume\";\nexport function call() { return consumeCall(); }\n",
    );

    // Act
    let output = project.run_cli();

    // Assert
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(diagnostics.contains("TS7023"), "{diagnostics}");
}
