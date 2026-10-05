use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-project-root-dirs-resolution-{}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("the temporary project directory can be created");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, relative_path: &str, contents: &str) {
        let path = self.path.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("the source directory can be created");
        }
        fs::write(path, contents).expect("the project source can be written");
    }

    fn run_cli(&self, arguments: &[&std::ffi::OsStr]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .current_dir(self.path())
            .args(arguments)
            .output()
            .expect("the compiler CLI can be started")
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

// Pinned TypeScript fixture: compiler/pathMappingBasedModuleResolution6_node.ts.
// TS-Go 7.0.2 resolves both cross-root imports with rootDirs=["src","generated/src"].
#[test]
fn should_resolve_cross_root_imports_given_root_dirs_project_option_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "tsconfig.json",
        r#"{"files":["src/file1.ts"],"compilerOptions":{"target":"es2015","module":"commonjs","strict":false,"rootDirs":["src","generated/src"]}}"#,
    );
    project.write(
        "src/file1.ts",
        "import { x } from \"./project/file3\";\nconst value: number = x;\n",
    );
    project.write("src/file2/index.d.ts", "export let x: number;\n");
    project.write(
        "generated/src/project/file3.ts",
        "export { x } from \"../file2\";\n",
    );

    // Act
    let process = project.run_cli(&["--project".as_ref(), project.path().as_os_str()]);

    // Assert
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );
    assert!(process.status.success(), "{diagnostics}");
}
