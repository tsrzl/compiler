use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-transpile-local-type-query-{}",
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
            .expect("the TypeScript fixture can be written");
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

// Pinned fixture: transpile/declarationNotInScopeTypes.ts (`two`), with declaration=true and target=es6.
#[test]
fn should_emit_literal_return_type_given_local_type_query_when_emitting_declarations() {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"files":["input.ts"],"compilerOptions":{"declaration":true,"target":"es6"}}"#,
    );
    project.write(
        "input.ts",
        "export function two() {\n    const y = \"\";\n    return \"\" as typeof y;\n}\n",
    );

    // Act
    let output = project.run_cli();

    // Assert
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let declaration = fs::read_to_string(project.path().join("input.d.ts"))
        .expect("the declaration output can be read");
    assert!(
        declaration.contains("export declare function two(): \"\";"),
        "expected the local typeof query to resolve to the empty-string literal type, got {declaration:?}"
    );
}
