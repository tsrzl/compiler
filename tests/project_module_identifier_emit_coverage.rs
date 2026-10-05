use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let root =
            std::env::temp_dir().join(format!("tsrzl-module-identifier-{}", std::process::id()));
        fs::create_dir_all(&root).expect("the temporary project directory can be created");
        Self { root }
    }

    fn write(&self, name: &str, contents: &str) {
        fs::write(self.root.join(name), contents).expect("the project source can be written");
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// Pinned projects: projects/ModuleIdentifier/consume.ts and projects/baseline/emit.ts.
// Direct TS-Go 7.0.2 with --module commonjs emits a require for each relative import-equals.
#[test]
fn should_emit_module_identifier_as_commonjs_require_given_sibling_module_when_running_compiler_cli()
 {
    // Arrange
    let project = TemporaryProject::new();
    project.write(
        "consume.ts",
        "import M = require(\"./decl\");\n\nvar p : M.P;\nvar x1 = M.a;\n",
    );
    project.write(
        "decl.ts",
        "export interface P { x: number; y: number; }\nexport var a = 1;\n",
    );

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--module")
        .arg("commonjs")
        .arg(project.path().join("consume.ts"))
        .output()
        .expect("the compiler CLI can be started");
    let javascript = fs::read_to_string(project.path().join("consume.js"))
        .expect("the emitted JavaScript can be read");

    // Assert
    assert_eq!(
        javascript,
        "\"use strict\";\nObject.defineProperty(exports, \"__esModule\", { value: true });\nconst M = require(\"./decl\");\nvar p;\nvar x1 = M.a;\n",
        "the module identifier should emit as the pinned CommonJS require; diagnostics: {}{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );
    assert!(process.status.success());
}
