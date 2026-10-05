use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-project-non-relative-paths-{}-{name}",
            std::process::id(),
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
            fs::create_dir_all(parent).expect("the project source directory can be created");
        }
        fs::write(path, contents).expect("the project source file can be written");
    }

    fn run_cli(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .current_dir(self.path())
            .args(["--project", "."])
            .output()
            .expect("the compiler CLI can be started")
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

// Pinned project group: projects/non-relative/consume.ts and lib/bar/a.ts.
// TS 7 config uses paths because the former baseUrl option has been removed.
#[test]
fn should_resolve_non_relative_project_import_given_paths_pattern_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new("root-config");
    project.write(
        "tsconfig.json",
        r#"{"files":["consume.ts"],"compilerOptions":{"target":"es2015","module":"commonjs","paths":{"lib/*":["./lib/*"]}}}"#,
    );
    project.write(
        "consume.ts",
        "import { hello } from 'lib/bar/a';\nhello();\n",
    );
    project.write("lib/bar/a.ts", "export function hello(): void {}\n");

    // Act
    let output = project.run_cli();

    // Assert
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

// Pinned project group: projects/non-relative/consume.ts with a TS 7 paths configuration.
#[test]
fn should_resolve_inherited_path_pattern_given_project_import_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new("inherited-config");
    project.write(
        "configs/base/tsconfig.json",
        r#"{"compilerOptions":{"paths":{"lib/*":["./lib/*"]}}}"#,
    );
    project.write(
        "tsconfig.json",
        r#"{"extends":"./configs/base/tsconfig.json","files":["consume.ts"],"compilerOptions":{"target":"es2015","module":"commonjs"}}"#,
    );
    project.write(
        "consume.ts",
        "import { hello } from 'lib/bar/a';\nhello();\n",
    );
    project.write(
        "configs/base/lib/bar/a.ts",
        "export function hello(): void {}\n",
    );
    // A project-root-relative mapping would select this incompatible decoy.
    project.write("lib/bar/a.ts", "export const hello: number = 42;\n");

    // Act
    let output = project.run_cli();

    // Assert
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
