use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-library-target-{}-{name}",
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
            fs::create_dir_all(parent).expect("the project fixture directory can be created");
        }
        fs::write(path, contents).expect("the project fixture can be written");
    }

    fn run_cli(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .current_dir(self.path())
            .arg("--project")
            .arg(self.path())
            .output()
            .expect("the compiler CLI can be started")
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn should_report_missing_object_values_given_es5_library_when_compiling_project() {
    // TypeScript 7.0.2 case: conformance/es2017/useObjectValuesAndEntries2.ts.
    // With target ES2015 and lib ES5, Object.values reports TS2550.
    // Arrange
    let project = TemporaryProject::new("object-values");
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"es2015","lib":["es5"]},"files":["main.ts"]}"#,
    );
    project.write("main.ts", "const value = Object.values({ answer: 42 });");

    // Act
    let process = project.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );

    // Assert
    assert!(
        diagnostics.contains("TS2550"),
        "expected an ES2017 library diagnostic, got: {diagnostics}"
    );
}

#[test]
fn should_report_missing_object_entries_given_es5_library_when_compiling_project() {
    // TypeScript 7.0.2 case: conformance/es2017/useObjectValuesAndEntries2.ts.
    // With target ES2015 and lib ES5, Object.entries reports TS2550.
    // Arrange
    let project = TemporaryProject::new("object-entries");
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"es2015","lib":["es5"]},"files":["main.ts"]}"#,
    );
    project.write("main.ts", "const value = Object.entries({ answer: 42 });");

    // Act
    let process = project.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );

    // Assert
    assert!(
        diagnostics.contains("TS2550"),
        "expected an ES2017 library diagnostic, got: {diagnostics}"
    );
}

#[test]
fn should_accept_object_values_given_es2017_object_library_when_compiling_project() {
    // TypeScript 7.0.2 case: conformance/es2017/useObjectValuesAndEntries1.ts.
    // The ES2017.Object library declares Object.values independently of target ES2015.
    // Arrange
    let project = TemporaryProject::new("object-values-es2017");
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"es2015","lib":["es5","es2017.object"]},"files":["main.ts"]}"#,
    );
    project.write(
        "main.ts",
        "const values: number[] = Object.values({ answer: 42 });",
    );

    // Act
    let process = project.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );

    // Assert
    assert!(process.status.success(), "{diagnostics}");
}

#[test]
fn should_accept_object_entries_given_es2017_object_library_when_compiling_project() {
    // TypeScript 7.0.2 case: conformance/es2017/useObjectValuesAndEntries1.ts.
    // ES2017.Object types Object.entries as key/value tuples independently of target ES2015.
    // Arrange
    let project = TemporaryProject::new("object-entries-es2017");
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"es2015","lib":["es5","es2017.object"]},"files":["main.ts"]}"#,
    );
    project.write("main.ts", "Object.entries({ answer: 42 });");

    // Act
    let process = project.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );

    // Assert
    assert!(process.status.success(), "{diagnostics}");
}
