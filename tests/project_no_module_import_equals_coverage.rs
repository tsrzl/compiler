use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrzl-no-module-import-equals-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("the temporary project directory can be created");
        Self { root }
    }

    fn write(&self, relative_path: &str, contents: &str) {
        let path = self.root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("the fixture directory can be created");
        }
        fs::write(path, contents).expect("the fixture file can be written");
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// Pinned project: tests/cases/project/cantFindTheModule.json selects projects/NoModule/decl.ts.
// Direct TS-Go 7.0.2 reports TS2307 for the bare `baz` import assignment.
#[test]
fn should_report_missing_bare_import_assignment_given_no_module_project_input_when_running_compiler_cli()
 {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "decl.ts",
        "import modErr  = require(\"./foo/bar.tx\");\nimport modErr1 = require(\"baz\");\nimport modErr2 = require(\"./baz\");\n\n//import modErr1 = require(\"\\bar\");\n\n//import mod  = require(\"./foo/bar\");\n//import mod1 = require(\"../module paths/foo/bar\");\n//import mod2 = require(\"foo/bar\");\n\nexport function call ( ) {\n\treturn \"hello world\";\n}\n",
    );
    project.write("foo/bar.ts", "export const value = 1;\n");
    let input = project.path().join("decl.ts");

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg(&input)
        .output()
        .expect("the compiler CLI can be started");
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(
        diagnostics.contains("Cannot find module 'baz' or its corresponding type declarations."),
        "the unresolved bare import assignment should report TS2307, got: {diagnostics}"
    );
}
