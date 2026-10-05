use std::fs;
use std::path::PathBuf;
use std::process::Command;

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-project-repeated-module-declaration-import-{}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("the temporary project directory can be created");
        Self { path }
    }

    fn write(&self, relative_path: &str, contents: &str) {
        fs::write(self.path.join(relative_path), contents)
            .expect("the project source file can be written");
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

// Pinned project: projects/declarations_MultipleTimesImport/{useModule.ts,m4.ts}.
#[test]
fn should_emit_one_import_given_repeated_module_imports_when_emitting_declarations() {
    // Arrange
    // Based on projects/declarations_MultipleTimesImport/{useModule.ts,m4.ts}.
    // ESM imports isolate declaration deduplication from that fixture's import-equals syntax.
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"files":["useModule.ts"],"compilerOptions":{"declaration":true}}"#,
    );
    project.write(
        "m4.ts",
        "export class D {}\nexport const x: D = new D();\nexport function foo() { return new D(); }\n",
    );
    project.write(
        "useModule.ts",
        "import * as m4 from \"./m4\";\nexport const x4 = m4.x;\nexport const d4 = m4.D;\nexport const f4 = m4.foo();\nimport * as multiImport_m4 from \"./m4\";\nexport const useMultiImport_m4_x4 = multiImport_m4.x;\nexport const useMultiImport_m4_d4 = multiImport_m4.D;\nexport const useMultiImport_m4_f4 = multiImport_m4.foo();\n",
    );

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .current_dir(&project.path)
        .args(["--project", "."])
        .output()
        .expect("the compiler CLI can be started");
    assert!(
        output.status.success(),
        "the declaration project should compile; stdout: {}; stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let declaration = fs::read_to_string(project.path.join("useModule.d.ts"))
        .expect("the declaration output can be read");

    // Assert
    assert_eq!(
        declaration.matches("import * as").count(),
        1,
        "repeated imports from one module should produce one declaration import; CLI: {}{}; declaration: {declaration:?}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
