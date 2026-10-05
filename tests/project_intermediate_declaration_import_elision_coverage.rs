use std::fs;
use std::path::PathBuf;
use std::process::Command;

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-project-intermediate-declaration-import-elision-{}",
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

// Pinned project: projects/declarations_MultipleTimesMultipleImport/{useModule.ts,m5.ts,m4.ts}.
#[test]
fn should_elide_intermediate_module_import_given_exported_inferred_type_when_emitting_declarations()
{
    // Arrange
    // Based on projects/declarations_MultipleTimesMultipleImport/{useModule.ts,m5.ts,m4.ts}.
    // ESM imports isolate declaration import elision from import-equals syntax.
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
        "m5.ts",
        "import { D } from \"./m4\";\nexport function foo2() { return new D(); }\n",
    );
    project.write(
        "useModule.ts",
        "import * as m4 from \"./m4\";\nexport const x4 = m4.x;\nexport const d4 = m4.D;\nexport const f4 = m4.foo();\nimport { foo2 } from \"./m5\";\nexport const d = foo2();\n",
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
    assert!(
        declaration.contains("export declare const d: m4.D;")
            && !declaration.contains("from \"./m5\""),
        "the exported type should use m4 directly without importing intermediate m5; CLI: {}{}; declaration: {declaration:?}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
