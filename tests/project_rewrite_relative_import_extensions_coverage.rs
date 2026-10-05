use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrzl-rewrite-relative-import-extensions-{}",
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

// Pinned TypeScript fixture: conformance/externalModules/rewriteRelativeImportExtensions/emit.ts.
// TS-Go 7.0.2 rewrites the relative .ts import to .js in emitted JavaScript.
#[test]
fn should_rewrite_relative_typescript_import_given_rewrite_option_when_emitting_javascript() {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"esnext","module":"preserve","moduleResolution":"bundler","verbatimModuleSyntax":true,"rewriteRelativeImportExtensions":true,"outDir":"dist"},"files":["main.ts","dep.ts"]}"#,
    );
    project.write(
        "main.ts",
        "import { answer } from './dep.ts';\nexport const result: number = answer;\n",
    );
    project.write("dep.ts", "export const answer: number = 42;\n");

    // Act
    let output = project.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    assert_eq!(
        fs::read_to_string(project.path().join("dist/main.js"))
            .expect("the emitted JavaScript can be read"),
        "import { answer } from \"./dep.js\";\nexport const result = answer;\n"
    );
}
