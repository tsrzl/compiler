use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let root =
            std::env::temp_dir().join(format!("tsrzl-export-as-namespace-{}", std::process::id()));
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
        let _ = fs::remove_dir_all(&self.root);
    }
}

// Pinned project cases: projects/declarations_ExportNamespace/decl.d.ts and projects/declarations_ExportNamespace/useModule.ts.
#[test]
fn should_resolve_ambient_global_namespace_type_given_export_as_namespace_when_emitting_declarations()
 {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"files":["decl.d.ts","useModule.ts"],"compilerOptions":{"declaration":true,"strict":false}}"#,
    );
    project.write(
        "decl.d.ts",
        "export interface A { b: number; }\nexport as namespace moduleA;",
    );
    project.write(
        "useModule.ts",
        "namespace moduleB { export interface IUseModuleA { a: moduleA.A; } }",
    );

    // Act
    let output = project.run_cli();

    // Assert
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let declaration = fs::read_to_string(project.path().join("useModule.d.ts"))
        .expect("the declaration output can be read");
    assert!(
        declaration.contains("a: moduleA.A;"),
        "expected the ambient global namespace type to be preserved, got {declaration:?}"
    );
}
