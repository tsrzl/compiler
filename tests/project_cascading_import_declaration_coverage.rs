use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrzl-cascading-import-declaration-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("the temporary project directory can be created");
        Self { root }
    }

    fn write(&self, name: &str, source: &str) {
        fs::write(self.root.join(name), source).expect("the TypeScript source can be written");
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

// Pinned project: projects/declarations_CascadingImports/useModule.ts and m4.ts.
// The equivalent relative-import chain keeps the imported type binding in its intermediate declaration.
#[test]
fn should_preserve_cascaded_type_import_given_exported_class_property_when_emitting_declarations() {
    // Arrange
    let project = TemporaryProject::new();
    project.write("m4.ts", "export class D {}\n");
    project.write(
        "m1.ts",
        "import { D } from \"./m4\";\nexport class V { c: D = new D(); }\n",
    );
    project.write(
        "m2.ts",
        "import { V } from \"./m1\";\nexport const value: V = new V();\n",
    );

    // Act
    let process = Command::new(env!("CARGO_BIN_EXE_tsrzl"))
        .current_dir(project.path())
        .args(["--module", "commonjs", "--declaration", "m2.ts"])
        .output()
        .expect("the compiler CLI can be started");

    // Assert
    assert!(
        process.status.success(),
        "{}{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );
    let declaration = fs::read_to_string(project.path().join("m1.d.ts"))
        .expect("the intermediate declaration file can be read");
    assert!(
        declaration.contains("import { D } from \"./m4\";"),
        "the class property type should retain its imported binding; got {declaration:?}"
    );
}
