use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-project-no-emit-option-{}",
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
            fs::create_dir_all(parent).expect("the fixture directory can be created");
        }
        fs::write(path, contents).expect("the project file can be written");
    }

    fn run_cli(&self, arguments: &[&std::ffi::OsStr]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .current_dir(self.path())
            .args(arguments)
            .output()
            .expect("the compiler CLI can be started")
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

// Pinned TypeScript fixture: compiler/compilerOptionsOutDirAndNoEmit.ts.
// TS-Go 7.0.2 baseline: submodule/compiler/compilerOptionsOutDirAndNoEmit.types.
#[test]
fn should_suppress_javascript_output_given_no_emit_project_configuration_when_running_compiler_cli()
{
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"files":["a.ts"],"compilerOptions":{"target":"es2015","outDir":"outDir","noEmit":true}}"#,
    );
    project.write("a.ts", "class C {}\n");

    // Act
    let process = project.run_cli(&["--project".as_ref(), project.path().as_os_str()]);

    // Assert
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );
    assert!(process.status.success(), "{diagnostics}");
    assert!(
        !project.path().join("outDir/a.js").exists(),
        "{diagnostics}"
    );
}
