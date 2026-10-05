use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrzl-project-reference-emit-{}",
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
            fs::create_dir_all(parent).expect("the fixture directory can be created");
        }
        fs::write(path, contents).expect("the TypeScript fixture can be written");
    }

    fn compile_root(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .current_dir(self.path())
            .args([
                "--target",
                "es2015",
                "--declaration",
                "--out-dir",
                "out",
                "src/ts/foo/foo.ts",
            ])
            .output()
            .expect("the compiler CLI can be started")
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// Pinned TypeScript project fixture: projects/ReferenceResolution/src/ts/foo/foo.ts.
// It references projects/ReferenceResolution/bar/bar.ts through a relative triple-slash path.
// TS-Go 7.0.2 with --target es2015 --declaration emits out/bar/bar.d.ts when foo.ts is the root.
#[test]
fn should_emit_declaration_for_referenced_source_given_triple_slash_path_when_compiling_root() {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "src/ts/foo/foo.ts",
        "/// <reference path=\"../../../bar/bar.ts\" />\n\nclass foo {\n}",
    );
    project.write(
        "bar/bar.ts",
        "/// <reference path=\"../src/ts/foo/foo.ts\" />\n// This is bar.ts\nclass bar {\n}",
    );

    // Act
    let output = project.compile_root();

    // Assert
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "{diagnostics}");
    assert!(
        project.path().join("out/bar/bar.d.ts").is_file(),
        "the referenced bar source should be included in declaration emit"
    );
}
