use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-project-multiple-levels-{}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("the temporary project directory can be created");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, relative_path: &str, contents: &str) {
        let path = self.path.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("the nested source directory can be created");
        }
        fs::write(path, contents).expect("the TypeScript source file can be written");
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

// Pinned TypeScript fixture: projects/MultipleLevels/B/B.ts.
// TS-Go 7.0.2 baseline: compiling B/B.ts resolves both relative import-equals dependencies.
#[test]
fn should_resolve_nested_import_equals_dependencies_given_multilevel_project_when_running_compiler_cli()
 {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "B/B.ts",
        "import A = require(\"../A/A\");\nexport const answer: number = A.answer;\n",
    );
    project.write("A/A.ts", "export const answer: number = 42;\n");

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .current_dir(project.path())
        .args(["--module", "commonjs", "B/B.ts"])
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );
}
