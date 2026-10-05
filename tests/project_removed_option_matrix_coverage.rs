use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrzl-removed-option-{}-{name}",
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

    fn run_cli(&self) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .current_dir(self.path())
            .args(["--project", "."])
            .output()
            .expect("the compiler CLI can be started")
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn run_project_with_options(name: &str, compiler_options: &str) -> String {
    let project = TemporaryProject::new(name);
    project.write(
        "tsconfig.json",
        &format!(r#"{{"files":["main.ts"],"compilerOptions":{{{compiler_options}}}}}"#),
    );
    project.write("main.ts", "export const answer = 42;");
    let output = project.run_cli();

    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

// Pinned TypeScript-Go source: internal/compiler/program.go removed-option diagnostics.
#[test]
fn should_report_removed_node10_resolution_given_project_config_when_running_cli() {
    // Arrange
    let compiler_options = r#""moduleResolution":"node10""#;

    // Act
    let diagnostics = run_project_with_options("node10-resolution", compiler_options);

    // Assert
    assert!(
        diagnostics.contains("TS5108") && diagnostics.contains("moduleResolution=node10"),
        "expected the TS5108 removed node10 module-resolution diagnostic, got {diagnostics:?}"
    );
}

#[test]
fn should_report_removed_amd_module_given_project_config_when_running_cli() {
    // Arrange
    let compiler_options = r#""module":"amd""#;

    // Act
    let diagnostics = run_project_with_options("amd-module", compiler_options);

    // Assert
    assert!(
        diagnostics.contains("TS5108") && diagnostics.contains("module=AMD"),
        "expected the TS5108 removed AMD module diagnostic, got {diagnostics:?}"
    );
}

#[test]
fn should_report_removed_system_module_given_project_config_when_running_cli() {
    // Arrange
    let compiler_options = r#""module":"system""#;

    // Act
    let diagnostics = run_project_with_options("system-module", compiler_options);

    // Assert
    assert!(
        diagnostics.contains("TS5108") && diagnostics.contains("module=System"),
        "expected the TS5108 removed System module diagnostic, got {diagnostics:?}"
    );
}

#[test]
fn should_report_removed_umd_module_given_project_config_when_running_cli() {
    // Arrange
    let compiler_options = r#""module":"umd""#;

    // Act
    let diagnostics = run_project_with_options("umd-module", compiler_options);

    // Assert
    assert!(
        diagnostics.contains("TS5108") && diagnostics.contains("module=UMD"),
        "expected the TS5108 removed UMD module diagnostic, got {diagnostics:?}"
    );
}

#[test]
fn should_report_removed_always_strict_false_given_project_config_when_running_cli() {
    // Arrange
    let compiler_options = r#""alwaysStrict":false"#;

    // Act
    let diagnostics = run_project_with_options("always-strict-false", compiler_options);

    // Assert
    assert!(
        diagnostics.contains("TS5108") && diagnostics.contains("alwaysStrict=false"),
        "expected the TS5108 removed alwaysStrict=false diagnostic, got {diagnostics:?}"
    );
}

#[test]
fn should_report_removed_es_module_interop_false_given_project_config_when_running_cli() {
    // Arrange
    let compiler_options = r#""esModuleInterop":false"#;

    // Act
    let diagnostics = run_project_with_options("es-module-interop-false", compiler_options);

    // Assert
    assert!(
        diagnostics.contains("TS5108") && diagnostics.contains("esModuleInterop=false"),
        "expected the TS5108 removed esModuleInterop=false diagnostic, got {diagnostics:?}"
    );
}

#[test]
fn should_report_removed_synthetic_default_imports_false_given_project_config_when_running_cli() {
    // Arrange
    let compiler_options = r#""allowSyntheticDefaultImports":false"#;

    // Act
    let diagnostics = run_project_with_options("synthetic-default-imports-false", compiler_options);

    // Assert
    assert!(
        diagnostics.contains("TS5108")
            && diagnostics.contains("allowSyntheticDefaultImports=false"),
        "expected the TS5108 removed allowSyntheticDefaultImports=false diagnostic, got {diagnostics:?}"
    );
}

#[test]
fn should_report_removed_downlevel_iteration_given_project_config_when_running_cli() {
    // Arrange
    let compiler_options = r#""downlevelIteration":true"#;

    // Act
    let diagnostics = run_project_with_options("downlevel-iteration", compiler_options);

    // Assert
    assert!(
        diagnostics.contains("TS5102") && diagnostics.contains("downlevelIteration"),
        "expected the TS5102 removed downlevelIteration diagnostic, got {diagnostics:?}"
    );
}
