use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrzl-default-out-dir-exclusion-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("the temporary project directory can be created");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn write(&self, relative_path: &str, contents: &str) {
        let path = self.root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("the project directory can be created");
        }
        fs::write(path, contents).expect("the project file can be written");
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// Pinned TypeScript project fixture: projects/projectOption/DefaultExcludeNodeModulesAndOutDir/a.ts.
// TS-Go 7.0.2 excludes the configured outDir from implicit root discovery.
#[test]
fn should_exclude_output_directory_from_implicit_roots_given_out_dir_option_when_running_compiler_cli()
 {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"outDir":"OutDir","declaration":true}}"#,
    );
    project.write("a.ts", "var test = 10;\n");
    project.write(
        "OutDir/generated.ts",
        "const generated: number = \"this file must not be checked\";\n",
    );

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .current_dir(project.path())
        .args(["--project", "tsconfig.json"])
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    let output = format!(
        "{}{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );
    assert!(process.status.success(), "{output}");
    assert!(
        !output.contains("TS2322"),
        "the output directory was checked: {output}"
    );
    assert!(project.path().join("OutDir/a.d.ts").exists());
}
