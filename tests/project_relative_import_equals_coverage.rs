use std::fs;
use std::path::PathBuf;
use std::process::Command;

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-project-relative-global-{}",
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

// Pinned case: projects/relative-global/consume.ts with its relative-global/decl.ts dependency.
#[test]
fn should_compile_relative_import_equals_dependency_given_project_root_when_running_compiler_cli() {
    // Arrange
    // Pinned fixture: projects/relative-global/consume.ts and decl.ts.
    // TS-Go project options: target=es2015, module=commonjs, noImplicitAny=false.
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"files":["consume.ts"],"compilerOptions":{"target":"ES2015","module":"commonjs","noImplicitAny":false,"outDir":"dist"}}"#,
    );
    project.write(
        "consume.ts",
        "import decl = require(\"./decl\");\nvar str = decl.call();\ndeclare function fail();\nif (str !== \"success\") { fail(); }\n",
    );
    project.write(
        "decl.ts",
        "export function call() { return \"success\"; }\nexport var x = 1;\n",
    );

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .current_dir(&project.path)
        .args(["--project", "."])
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        output.status.success(),
        "the relative import-equals project should compile; stdout: {}; stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
