use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-project-package-config-exports-{}",
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

// Pinned TypeScript fixture: compiler/tsconfigExtendsPackageJsonExportsWildcard.ts.
// Pinned TypeScript fixture: compiler/tsconfigExtendsPackageJsonExportsWildcard.ts.
// This project distills its wildcard package-export path and observes the inherited strictNullChecks option.
#[test]
fn should_resolve_package_extended_config_through_exports_wildcard_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "node_modules/foo/package.json",
        r#"{"name":"foo","version":"1.0.0","exports":{"./*.json":"./configs/*.json"}}"#,
    );
    project.write(
        "node_modules/foo/configs/strict.json",
        r#"{"compilerOptions":{"strict":true,"strictNullChecks":true}}"#,
    );
    project.write("tsconfig.json", r#"{"extends":"foo/strict.json"}"#);
    project.write(
        "index.ts",
        "let value: string | undefined = undefined; const result: string = value;",
    );

    // Act
    let process = project.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );

    // Assert
    assert!(diagnostics.contains("TS2322"), "{diagnostics}");
}
