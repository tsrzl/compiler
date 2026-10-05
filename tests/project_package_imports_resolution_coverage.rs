use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrzl-package-imports-resolution-{}-{name}",
            std::process::id(),
        ));
        fs::create_dir_all(&root).expect("the project directory can be created");
        Self { root }
    }

    fn write(&self, relative_path: &str, contents: &str) {
        let path = self.root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("the source directory can be created");
        }
        fs::write(path, contents).expect("the project file can be written");
    }

    fn run_cli(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .arg("--project")
            .arg(&self.root)
            .output()
            .expect("the compiler CLI can be started")
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// Pinned TypeScript fixture: conformance/node/nodeModulesPackageImports.ts.
// TS-Go 7.0.2 resolves #answer through package.json imports to feature.ts.
#[test]
fn should_resolve_package_import_map_given_hash_specifier_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new("exact");
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"es2022","module":"preserve","moduleResolution":"bundler","strict":true,"noEmit":true},"files":["index.ts","feature.ts"]}"#,
    );
    project.write(
        "package.json",
        r##"{"name":"tsrzl-package-imports-resolution","private":true,"type":"module","imports":{"#answer":"./feature.js"}}"##,
    );
    project.write(
        "index.ts",
        "import { answer } from \"#answer\";\nconst output: string = answer;\n",
    );
    project.write("feature.ts", "export const answer: number = 42;\n");

    // Act
    let output = project.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(
        diagnostics.contains("TS2322"),
        "the mapped module's number type should reach the assignment check, got {diagnostics:?}"
    );
    assert!(
        !diagnostics.contains("TS2307"),
        "the package imports mapping should resolve #answer, got {diagnostics:?}"
    );
}

// Pinned TypeScript fixture: conformance/node/nodeModulesPackageImportsRootWildcard.ts.
// TS-Go 7.0.2 substitutes the wildcard and resolves the .js specifier to bar.ts.
#[test]
fn should_resolve_package_import_wildcard_given_nested_hash_specifier_when_running_compiler_cli() {
    // Arrange
    let project = TemporaryProject::new("wildcard");
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"es2022","module":"preserve","moduleResolution":"bundler","strict":true,"noEmit":true},"files":["index.ts","src/features/bar.ts"]}"#,
    );
    project.write(
        "package.json",
        r##"{"name":"tsrzl-package-imports-resolution","private":true,"type":"module","imports":{"#/*":"./src/*"}}"##,
    );
    project.write(
        "index.ts",
        "import { bar } from \"#/features/bar.js\";\nconst output: number = bar;\n",
    );
    project.write(
        "src/features/bar.ts",
        "export const bar: string = \"bar\";\n",
    );

    // Act
    let output = project.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(
        diagnostics.contains("TS2322"),
        "the wildcard target's string type should reach the assignment check, got {diagnostics:?}"
    );
    assert!(
        !diagnostics.contains("TS2307"),
        "the package imports wildcard should resolve the nested # specifier, got {diagnostics:?}"
    );
}
