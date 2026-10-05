use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrzl-native-extension-module-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("the temporary project directory can be created");
        Self { root }
    }

    fn write(&self, relative_path: &str, contents: &str) {
        let path = self.root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("the fixture directory can be created");
        }
        fs::write(path, contents).expect("the fixture file can be written");
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

#[test]
fn should_resolve_arbitrary_native_extension_declaration_given_allow_arbitrary_extensions_when_running_node18_compiler_cli()
 {
    // Pinned TypeScript fixture: conformance/nonjsExtensions/declarationFilesForNodeNativeModules.ts.
    // Its node18 + allowArbitraryExtensions=true baseline emits main.js after resolving
    // ./dir/native.node to dir/native.d.node.ts.
    // Arrange
    let project = TemporaryProject::new();
    project.write("package.json", r#"{"type":"module"}"#);
    project.write("dir/package.json", r#"{"type":"commonjs"}"#);
    project.write(
        "dir/native.d.node.ts",
        "export function doNativeThing(flag: string): unknown;\n",
    );
    project.write(
        "main.ts",
        "import mod = require(\"./dir/native.node\");\nmod.doNativeThing(\"good\");\n",
    );
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"es2020","module":"node18","allowArbitraryExtensions":true},"files":["main.ts"]}"#,
    );

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .arg("--project")
        .arg(project.path())
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    assert_eq!(
        fs::read_to_string(project.path().join("main.js"))
            .expect("the emitted JavaScript can be read"),
        "import { createRequire as _createRequire } from \"module\";\nconst __require = _createRequire(import.meta.url);\nconst mod = __require(\"./dir/native.node\");\nmod.doNativeThing(\"good\");\n"
    );
}
