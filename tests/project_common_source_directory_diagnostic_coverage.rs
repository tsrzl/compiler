use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryWorkspace {
    root: PathBuf,
}

impl TemporaryWorkspace {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrzl-common-source-directory-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("the workspace directory can be created");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn write(&self, relative_path: &str, contents: &str) {
        let path = self.root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("the source directory can be created");
        }
        fs::write(path, contents).expect("the project fixture can be written");
    }

    fn project_path(&self) -> PathBuf {
        self.path().join("outputdir_module_multifolder")
    }

    fn run_cli(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .current_dir(self.project_path())
            .args(["--project", "."])
            .output()
            .expect("the compiler CLI can be started")
    }
}

impl Drop for TemporaryWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// Pinned project inputs: projects/outputdir_module_multifolder/test.ts and projects/outputdir_module_multifolder_ref/m2.ts.
#[test]
fn should_report_common_source_directory_outside_project_given_out_dir_without_root_dir_when_compiling_project()
 {
    // Arrange
    let workspace = TemporaryWorkspace::new();
    workspace.write(
        "outputdir_module_multifolder/tsconfig.json",
        r#"{"files":["test.ts"],"compilerOptions":{"strict":false,"declaration":true,"outDir":"outdir/simple"}}"#,
    );
    workspace.write(
        "outputdir_module_multifolder/test.ts",
        "import { m2_c1 } from \"../outputdir_module_multifolder_ref/m2\";\nexport const a3 = m2_c1;",
    );
    workspace.write(
        "outputdir_module_multifolder_ref/m2.ts",
        "export class m2_c1 {}",
    );

    // Act
    let output = workspace.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(
        diagnostics.contains("TS5011"),
        "expected TS5011 for a common source directory outside the project root, got {diagnostics:?}"
    );
}
