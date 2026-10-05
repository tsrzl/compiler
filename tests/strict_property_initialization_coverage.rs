use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

struct TempProject {
    root: PathBuf,
}

impl TempProject {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrzl-strict-property-initialization-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("the temporary project directory can be created");
        fs::write(root.join("input.ts"), "class C { value: number; }")
            .expect("the TypeScript input can be written");
        fs::write(
            root.join("tsconfig.json"),
            r#"{"compilerOptions":{"target":"ES2015","strict":true},"files":["input.ts"]}"#,
        )
        .expect("the project configuration can be written");

        Self { root }
    }

    fn compile(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .arg("--project")
            .arg(&self.root)
            .output()
            .expect("the compiler CLI can be started")
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn should_report_uninitialized_property_given_strict_project_option_when_compiling_project() {
    // TypeScript 7.0.2 case: conformance/classes/propertyMemberDeclarations/strictPropertyInitialization.ts.
    // Its strict=true configuration reports TS2564 for a required property with no initializer.
    // Arrange
    let project = TempProject::new();

    // Act
    let output = project.compile();

    // Assert
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("TS2564:"),
        "strict property initialization must report TS2564, got: {stderr}"
    );
}
