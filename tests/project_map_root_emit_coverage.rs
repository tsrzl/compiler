use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let root =
            std::env::temp_dir().join(format!("tsrzl-project-map-root-{}", std::process::id()));
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

// Pinned project case: projects/outputdir_simple/test.ts; project/mapRootRelativePathSimpleSpecifyOutputDirectory.json.
#[test]
fn should_emit_map_root_source_mapping_url_given_out_dir_and_source_map_when_running_compiler_cli()
{
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"files":["test.ts"],"compilerOptions":{"strict":false,"declaration":true,"sourceMap":true,"mapRoot":"../mapFiles","outDir":"outdir/simple"}}"#,
    );
    project.write(
        "test.ts",
        "/// <reference path='m1.ts'/>\nvar a1 = 10;\nclass c1 { public p1: number; }\nvar instance1 = new c1();\nfunction f1() { return instance1; }",
    );
    project.write("m1.ts", "var m1_a1 = 10;");

    // Act
    let output = project.run_cli();

    // Assert
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let javascript = fs::read_to_string(project.path().join("outdir/simple/test.js"))
        .expect("the emitted JavaScript can be read");
    assert!(
        javascript.contains("//# sourceMappingURL=../../../mapFiles/test.js.map"),
        "expected the mapRoot-relative source map URL, got {javascript:?}"
    );
}
