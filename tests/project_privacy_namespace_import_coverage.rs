use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new(context: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrzl-project-privacy-namespace-import-{context}-{}",
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

fn run_compiler(project: &TemporaryProject) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .current_dir(project.path())
        .args(["--project", "."])
        .output()
        .expect("the compiler CLI can be started");
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

// Pinned project fixture: projects/privacyCheck-ImportInParent/test.ts.
// TS-Go 7.0.2 reports TS1147 for import-equals inside the exported namespace.
#[test]
fn should_report_ts1147_given_import_equals_inside_exported_namespace_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new("exported");
    project.write(
        "tsconfig.json",
        r#"{"files":["test.ts","external.ts"],"compilerOptions":{"module":"commonjs","target":"es2015","noEmit":true,"paths":{"*":["./*"]}}}"#,
    );
    project.write(
        "test.ts",
        "export namespace parent { export import api = require(\"external\"); export const value = api.value; }\n",
    );
    project.write("external.ts", "export const value = 1;\n");

    // Act
    let diagnostics = run_compiler(&project);

    // Assert
    assert!(
        diagnostics.contains("TS1147"),
        "the project should report TS1147 for import-equals inside an exported namespace, got {diagnostics:?}"
    );
}

// Pinned project fixture: projects/privacyCheck-InsideModule/testGlo.ts.
// TS-Go 7.0.2 reports TS1147 for import-equals inside the global namespace.
#[test]
fn should_report_ts1147_given_import_equals_inside_global_namespace_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new("global");
    project.write(
        "tsconfig.json",
        r#"{"files":["test.ts","external.ts"],"compilerOptions":{"module":"commonjs","target":"es2015","noEmit":true,"paths":{"*":["./*"]}}}"#,
    );
    project.write(
        "test.ts",
        "namespace globalScope { export import api = require(\"external\"); export const value = api.value; }\n",
    );
    project.write("external.ts", "export const value = 1;\n");

    // Act
    let diagnostics = run_compiler(&project);

    // Assert
    assert!(
        diagnostics.contains("TS1147"),
        "the project should report TS1147 for import-equals inside a global namespace, got {diagnostics:?}"
    );
}
