use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-project-resolution-gap-{}",
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

// Pinned TypeScript fixture: conformance/moduleResolution/typesVersions.multiFile.ts.
// TS-Go 7.0.2 baseline: submodule/conformance/typesVersions.multiFile.types.
#[test]
fn should_resolve_versioned_package_types_given_types_versions_mapping_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"files":["main.ts"],"compilerOptions":{"target":"esnext","module":"commonjs"}}"#,
    );
    project.write("main.ts", "import { a } from 'ext';\nconst aa: number = a;");
    project.write(
        "node_modules/ext/package.json",
        r#"{"name":"ext","version":"1.0.0","types":"index","typesVersions":{">=3.1.0-0":{"*": ["ts3.1/*"]}}}"#,
    );
    project.write("node_modules/ext/index.d.ts", "export const a: string;");
    project.write(
        "node_modules/ext/ts3.1/index.d.ts",
        "export const a: number;",
    );

    // Act
    let process = project.run_cli(&["--project".as_ref(), project.path().as_os_str()]);

    // Assert
    assert!(
        process.status.success(),
        "{}{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );
}
