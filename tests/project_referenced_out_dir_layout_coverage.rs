use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrzl-referenced-out-dir-layout-{}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("ref")).expect("the project source directory can be created");
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

// Pinned project inputs: projects/outputdir_subfolder/test.ts, projects/outputdir_subfolder/ref/m1.ts, and projects/outputdir_mixed_subfolder/ref/m1.ts.
#[test]
fn should_preserve_referenced_source_subdirectory_given_out_dir_project_option_when_running_compiler_cli()
 {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"files":["test.ts"],"compilerOptions":{"strict":false,"declaration":true,"outDir":"outdir/simple"}}"#,
    );
    project.write(
        "test.ts",
        "/// <reference path='ref/m1.ts'/>\nvar a1 = 10;\nclass c1 { public p1: number; }\nvar instance1 = new c1();\nfunction f1() { return instance1; }",
    );
    project.write("ref/m1.ts", "var m1_a1 = 10;");

    // Act
    let output = project.run_cli();

    // Assert
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        project.path().join("outdir/simple/ref/m1.js").exists(),
        "expected the referenced source at outdir/simple/ref/m1.js"
    );
}
