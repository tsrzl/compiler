use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let root =
            std::env::temp_dir().join(format!("tsrzl-project-ext-int-ext-{}", std::process::id()));
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
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// Pinned project fixtures: projects/ext-int-ext/{internal2.ts,external2.ts}.
// TS-Go 7.0.2 reports TS1147 for the namespace-local import-equals statement.
#[test]
fn should_report_ts1147_given_namespace_import_equals_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"files":["internal2.ts","external2.ts"],"compilerOptions":{"module":"commonjs","target":"es2015","noEmit":true}}"#,
    );
    project.write(
        "internal2.ts",
        "namespace outer {\n import g = require(\"external2\");\n export var a = g.square(5);\n export var b = \"foo\";\n}\n",
    );
    project.write(
        "external2.ts",
        "export function square(x: number) { return x * x; }\n",
    );

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .current_dir(project.path())
        .args(["--project", "."])
        .output()
        .expect("the compiler CLI can be started");
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(
        diagnostics.contains("TS1147"),
        "the project should report TS1147 for import-equals inside a namespace, got {diagnostics:?}"
    );
}
