use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("tsrzl-project-type-roots-{}", std::process::id()));
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

// Pinned TypeScript input: conformance/references/library-reference-13.ts.
// Pinned TS-Go baseline: testdata/baselines/reference/submodule/conformance/library-reference-13.types.
// The explicit files list mirrors the fixture's noImplicitReferences runner setting.
#[test]
fn should_resolve_global_type_from_type_package_given_types_project_option_when_running_compiler_cli()
 {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"types":["jquery"],"typeRoots":["./types"]},"files":["consumer.ts"]}"#,
    );
    project.write("types/jquery/index.d.ts", "declare var $: { foo(): void };");
    project.write("consumer.ts", "$.foo();");

    // Act
    let process = project.run_cli();

    // Assert
    assert!(
        process.status.success(),
        "{}{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );
}
