use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-project-declaration-dir-out-dir-{}",
            std::process::id()
        ));
        fs::create_dir_all(path.join("subfolder"))
            .expect("the temporary project directory can be created");
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

// Pinned TypeScript case: projects/declarationDir/a.ts and projects/declarationDir/subfolder/b.ts; project/declarationDir2.json uses declarationDir=declarations and outDir=out.
#[test]
fn should_route_declarations_to_declaration_dir_given_out_dir_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"files":["a.ts","subfolder/b.ts","subfolder/c.ts"],"compilerOptions":{"rootDir":".","strict":false,"declaration":true,"declarationDir":"declarations","outDir":"out"}}"#,
    );
    project.write(
        "a.ts",
        "import {B} from './subfolder/b';\nexport class A { b: B; }",
    );
    project.write("subfolder/b.ts", "export class B {}\n");
    project.write(
        "subfolder/c.ts",
        "import {A} from '../a';\nexport class C { a: A; }",
    );

    // Act
    let output = project.run_cli();

    // Assert
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        project
            .path()
            .join("declarations/subfolder/b.d.ts")
            .exists()
    );
}
