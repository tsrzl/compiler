use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("tsrzl-project-cli-gap-{}", std::process::id()));
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

// Pinned TypeScript-Go baseline: tsc/ignoreConfig/mixing-project-and-files.js.
#[test]
fn should_reject_project_option_mixed_with_source_files_given_project_and_file_arguments_when_running_compiler_cli()
 {
    // Arrange
    let project = TemporaryProject::new();
    project.write("tsconfig.json", "{\n  \"include\": [\"src\"],\n}\n");
    project.write("src/a.ts", "export const a = 10;");
    project.write("c.ts", "export const c = 10;");

    // Act
    let process = project.run_cli(&[
        "-p".as_ref(),
        ".".as_ref(),
        "src/a.ts".as_ref(),
        "c.ts".as_ref(),
    ]);

    // Assert
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );
    assert!(!process.status.success(), "{diagnostics}");
    assert!(
        diagnostics.contains(
            "TS5042: Option 'project' cannot be mixed with source files on a command line."
        ),
        "{diagnostics}"
    );
}
