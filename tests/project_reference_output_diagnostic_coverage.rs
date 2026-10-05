use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

struct TemporaryWorkspace {
    root: PathBuf,
}

impl TemporaryWorkspace {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrzl-project-reference-output-diagnostic-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("the workspace directory can be created");
        Self { root }
    }

    fn write(&self, relative_path: &str, contents: &str) {
        let path = self.root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("the project directory can be created");
        }
        fs::write(path, contents).expect("the project fixture can be written");
    }

    fn application_config_path(&self) -> PathBuf {
        self.root.join("application/tsconfig.json")
    }

    fn run_cli(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .arg("--project")
            .arg(self.application_config_path())
            .output()
            .expect("the compiler CLI can be started")
    }
}

impl Drop for TemporaryWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// Pinned TypeScript-Go 7.0.2 reports TS6305 in internal/checker/checker.go
// when an import resolves to a composite reference with no emitted declaration.
#[test]
fn should_report_unbuilt_project_reference_given_composite_import_when_running_cli() {
    // Arrange
    let workspace = TemporaryWorkspace::new();
    workspace.write(
        "library/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"declaration":true,"outDir":"dist"},"files":["index.ts"]}"#,
    );
    workspace.write("library/index.ts", "export const answer: number = 42;\n");
    workspace.write(
        "application/tsconfig.json",
        r#"{"compilerOptions":{"composite":true,"target":"es2015","module":"commonjs","noEmit":true,"outDir":"dist"},"references":[{"path":"../library"}],"files":["main.ts"]}"#,
    );
    workspace.write(
        "application/main.ts",
        "import { answer } from '../library/index'; export const result: number = answer;\n",
    );

    // Act
    let output = workspace.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(
        diagnostics.contains("TS6305")
            && diagnostics.contains("has not been built from source file"),
        "the unbuilt project reference should report TS6305, got {diagnostics:?}"
    );
}
