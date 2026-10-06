use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    path: PathBuf,
}

impl TemporaryProject {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "tsrzl-project-package-types-{}-{name}",
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
            fs::create_dir_all(parent).expect("the package directory can be created");
        }
        fs::write(path, contents).expect("the project file can be written");
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

// Pinned TypeScript fixture: compiler/moduleResolution_packageJson_scopedPackage.ts.
// TS-Go 7.0.2 baseline: submodule/compiler/moduleResolution_packageJson_scopedPackage.types.
#[test]
fn should_resolve_scoped_package_types_given_package_json_types_field_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new("scoped-package");
    project.write(
        "tsconfig.json",
        r#"{"files":["main.ts"],"compilerOptions":{"target":"es2015","module":"commonjs"}}"#,
    );
    project.write(
        "main.ts",
        "import { x } from \"@foo/bar\";\nconst value: number = x;",
    );
    project.write(
        "node_modules/@foo/bar/package.json",
        r#"{"types":"types.d.ts"}"#,
    );
    project.write(
        "node_modules/@foo/bar/types.d.ts",
        "export const x: number;",
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

// Pinned project: projects/NodeModulesSearch/maxDepthIncreased/node_modules/@types/m4/entry.d.ts.
// TS-Go 7.0.2 resolves the @types declaration and reports TS2322 for assigning its number to string.
#[test]
fn should_report_ts2322_given_at_types_package_for_javascript_module_when_compiling_project() {
    // Arrange
    let project = TemporaryProject::new("javascript-at-types");
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"allowJs":true,"maxNodeModuleJsDepth":3,"module":"esnext","moduleResolution":"bundler","noEmit":true},"files":["root.ts"]}"#,
    );
    project.write(
        "root.ts",
        "import * as m4 from \"m4\"; const value: string = m4.foo;",
    );
    project.write(
        "node_modules/m4/package.json",
        r#"{"name":"m4","version":"1.0.0","main":"entry.js"}"#,
    );
    project.write(
        "node_modules/m4/entry.js",
        "exports.test = \"hello, world\";",
    );
    project.write(
        "node_modules/@types/m4/package.json",
        r#"{"types":"entry.d.ts","name":"m4","version":"1.0.0"}"#,
    );
    project.write(
        "node_modules/@types/m4/entry.d.ts",
        "export declare var foo: number;",
    );

    // Act
    let process = project.run_cli(&["--project".as_ref(), project.path().as_os_str()]);
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&process.stdout),
        String::from_utf8_lossy(&process.stderr)
    );

    // Assert
    assert!(
        diagnostics.contains("TS2322"),
        "the @types declaration should provide the numeric property type, got {diagnostics}"
    );
}
