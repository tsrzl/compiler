use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-project-referenced-ambient-module-{}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("the temporary project directory can be created");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, relative_path: &str, contents: &str) {
        fs::write(self.path.join(relative_path), contents)
            .expect("the project source file can be written");
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

// Pinned project group: projects/relative-global-ref/consume.ts and decl.d.ts.
// The ESM import form isolates reference loading from the fixture's import-equals syntax.
#[test]
fn should_resolve_referenced_ambient_module_given_named_import_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"files":["consume.ts"],"compilerOptions":{"target":"es2015","module":"commonjs"}}"#,
    );
    project.write(
        "consume.ts",
        "/// <reference path=\"decl.d.ts\" />\nimport { call } from \"decl\";\nconst result: string = call();\n",
    );
    project.write(
        "decl.d.ts",
        "declare module \"decl\" { export function call(): string; }\n",
    );

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
