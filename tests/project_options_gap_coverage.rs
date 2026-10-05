use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("tsrzl-project-options-gap-{}", std::process::id()));
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
        fs::write(path, contents).expect("the fixture file can be written");
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

// Pinned TypeScript fixture: compiler/noEmitOnError.ts.
// TS-Go 7.0.2 baseline: submodule/compiler/noEmitOnError.js (missing in original emit).
#[test]
fn should_suppress_javascript_output_given_type_error_and_no_emit_on_error_project_option_when_running_compiler_cli()
 {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"files":["main.ts"],"compilerOptions":{"target":"es2015","noEmitOnError":true}}"#,
    );
    project.write("main.ts", "const answer: number = \"wrong\";");

    // Act
    let process = project.run_cli(&["--project".as_ref(), project.path().as_os_str()]);

    // Assert
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );
    assert!(!process.status.success(), "{diagnostics}");
    assert!(!project.path().join("main.js").exists(), "{diagnostics}");
}
