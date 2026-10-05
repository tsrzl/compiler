use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrzl-custom-condition-resolution-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("the project directory can be created");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn write(&self, relative_path: &str, contents: &str) {
        let path = self.root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("the package directory can be created");
        }
        fs::write(path, contents).expect("the project fixture can be written");
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

// Pinned TypeScript fixture: conformance/moduleResolution/customConditions.ts.
// TS-Go 7.0.2 resolves the browser export condition for the reduced project.
#[test]
fn should_resolve_browser_export_condition_given_custom_conditions_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"es2015","module":"preserve","moduleResolution":"bundler","customConditions":["webpack","browser"],"resolvePackageJsonExports":true,"strict":true,"noEmit":true},"files":["index.ts"]}"#,
    );
    project.write(
        "index.ts",
        "import { browser } from 'lodash';\nconst selected: number = browser;\n",
    );
    project.write(
        "node_modules/lodash/package.json",
        r#"{"name":"lodash","version":"1.0.0","exports":{"browser":"./browser.ts","webpack":"./webpack.ts","default":"./index.ts"}}"#,
    );
    project.write(
        "node_modules/lodash/browser.ts",
        "export const browser: number = 1;\n",
    );
    project.write(
        "node_modules/lodash/webpack.ts",
        "export const webpack: number = 1;\n",
    );
    project.write(
        "node_modules/lodash/index.ts",
        "export const index: number = 1;\n",
    );

    // Act
    let output = project.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(
        output.status.success(),
        "the configured browser export condition should resolve, got {diagnostics:?}"
    );
}
