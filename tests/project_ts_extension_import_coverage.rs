use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let root =
            std::env::temp_dir().join(format!("tsrzl-ts-extension-import-{}", std::process::id()));
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
            .arg("--project")
            .arg(self.path())
            .output()
            .expect("the compiler CLI can be started")
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// Pinned TypeScript fixture: conformance/moduleResolution/allowImportingTsExtensions.ts.
// TS-Go 7.0.2 reports TS5097 when a .ts import is used without the option.
#[test]
fn should_report_ts_extension_import_given_option_disabled_when_running_cli() {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"es2015","module":"esnext","moduleResolution":"bundler","noEmit":true,"strict":true},"files":["main.ts"]}"#,
    );
    project.write(
        "main.ts",
        "import { answer } from './answer.ts';\nconst result: number = answer;\n",
    );
    project.write("answer.ts", "export const answer: number = 42;\n");

    // Act
    let output = project.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(
        diagnostics.contains("TS5097") && diagnostics.contains("allowImportingTsExtensions"),
        "the .ts import should require allowImportingTsExtensions, got {diagnostics:?}"
    );
}
