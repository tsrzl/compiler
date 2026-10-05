use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-project-circular-unused-import-{}",
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

// Pinned project: projects/CircularReferencing-2/{a,b,c}.ts, runner input a.ts.
// The unused import in a.ts closes the dependency cycle; ESM import syntax isolates emit behavior
// from the fixture's unsupported import-equals syntax.
#[test]
fn should_elide_unused_cycle_import_given_commonjs_project_when_emitting_javascript() {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"files":["a.ts"],"compilerOptions":{"target":"es2015","module":"commonjs"}}"#,
    );
    project.write(
        "a.ts",
        "import { C } from \"./c\";\nexport class A { constructor() {} }\n",
    );
    project.write(
        "b.ts",
        "import { A } from \"./a\";\nexport class B extends A { constructor() { super(); } }\n",
    );
    project.write(
        "c.ts",
        "import { B } from \"./b\";\nexport class C extends B { constructor() { super(); } }\n",
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
    let javascript = fs::read_to_string(project.path().join("a.js"))
        .expect("the root project's JavaScript output can be read");
    assert!(
        !javascript.contains("require(\"./c\")"),
        "an unused cycle-closing import should be elided, got {javascript:?}"
    );
}
