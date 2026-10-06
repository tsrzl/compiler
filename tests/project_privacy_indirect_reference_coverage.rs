use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrzl-project-privacy-indirect-reference-{}",
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
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// Pinned project fixtures: projects/privacyCheck-IndirectReference/{test.ts,externalModule.ts,indirectExternalModule.ts}.
// TS-Go 7.0.2 with module=commonjs, target=ES2015, and paths={"*":["./*"]} emits test.js with require("externalModule").
#[test]
fn should_emit_javascript_given_transitive_import_equals_project_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"files":["test.ts","externalModule.ts","indirectExternalModule.ts"],"compilerOptions":{"module":"commonjs","target":"es2015","paths":{"*":["./*"]},"outDir":"out"}}"#,
    );
    project.write(
        "test.ts",
        "import im1 = require(\"externalModule\");\nexport var x = im1.x;\n",
    );
    project.write(
        "externalModule.ts",
        "import im0 = require(\"indirectExternalModule\");\nexport var x = new im0.indirectClass();\n",
    );
    project.write(
        "indirectExternalModule.ts",
        "export class indirectClass {}\n",
    );

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .current_dir(project.path())
        .args(["--project", "."])
        .output()
        .expect("the compiler CLI can be started");
    let javascript = fs::read_to_string(project.path().join("out/test.js"));

    // Assert
    assert!(
        javascript
            .as_deref()
            .is_ok_and(|javascript| javascript.contains("require(\"externalModule\")")),
        "the project should emit test.js with its import-equals require, got {javascript:?}; compiler output: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
