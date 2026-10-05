use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new(case: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrzl-rewrite-relative-import-extensions-{}-{case}",
            std::process::id(),
        ));
        fs::create_dir_all(&root).expect("the project directory can be created");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn write(&self, relative_path: &str, contents: &str) {
        fs::write(self.root.join(relative_path), contents)
            .expect("the project fixture can be written");
    }

    fn run_cli(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .arg("--project")
            .arg(self.path())
            .output()
            .expect("the compiler CLI can be started")
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// Pinned TypeScript fixture: conformance/externalModules/rewriteRelativeImportExtensions/emit.ts.
// TS-Go 7.0.2 rewrites the relative .ts import to .js in emitted JavaScript.
#[test]
fn should_rewrite_relative_typescript_import_given_rewrite_option_when_emitting_javascript() {
    // Arrange
    let project = TemporaryProject::new("ts-import");
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"esnext","module":"preserve","moduleResolution":"bundler","verbatimModuleSyntax":true,"rewriteRelativeImportExtensions":true,"outDir":"dist"},"files":["main.ts","dep.ts"]}"#,
    );
    project.write(
        "main.ts",
        "import { answer } from \"./dep.ts\";\nexport const result: number = answer;\n",
    );
    project.write("dep.ts", "export const answer: number = 42;\n");

    // Act
    let output = project.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    assert_eq!(
        fs::read_to_string(project.path().join("dist/main.js"))
            .expect("the emitted JavaScript can be read"),
        "import { answer } from \"./dep.js\";\nexport const result = answer;\n"
    );
}

#[test]
fn should_rewrite_mts_import_to_mjs_given_rewrite_option_when_emitting_javascript() {
    // Arrange
    let project = TemporaryProject::new("mts-import");
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"esnext","module":"preserve","moduleResolution":"bundler","verbatimModuleSyntax":true,"rewriteRelativeImportExtensions":true,"outDir":"dist"},"files":["main.ts","dep.mts"]}"#,
    );
    project.write(
        "main.ts",
        "import { answer } from \"./dep.mts\";\nexport const result: number = answer;\n",
    );
    project.write("dep.mts", "export const answer: number = 42;\n");

    // Act
    let output = project.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    assert_eq!(
        fs::read_to_string(project.path().join("dist/main.js"))
            .expect("the emitted JavaScript can be read"),
        "import { answer } from \"./dep.mjs\";\nexport const result = answer;\n"
    );
}

// Pinned TypeScript fixture: conformance/externalModules/rewriteRelativeImportExtensions/emit.ts.
// TS-Go 7.0.2 rewrites the relative .cts import to .cjs in emitted JavaScript.
#[test]
fn should_rewrite_cts_import_to_cjs_given_rewrite_option_when_emitting_javascript() {
    // Arrange
    let project = TemporaryProject::new("cts-import");
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"esnext","module":"preserve","moduleResolution":"bundler","rewriteRelativeImportExtensions":true,"outDir":"dist"},"files":["main.ts","dep.cts"]}"#,
    );
    project.write(
        "main.ts",
        "import { answer } from \"./dep.cts\";\nexport const result: number = answer;\n",
    );
    project.write("dep.cts", "export const answer: number = 42;\n");

    // Act
    let output = project.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    assert_eq!(
        fs::read_to_string(project.path().join("dist/main.js"))
            .expect("the emitted JavaScript can be read"),
        "import { answer } from \"./dep.cjs\";\nexport const result = answer;\n"
    );
}

// Pinned TypeScript fixture: conformance/externalModules/rewriteRelativeImportExtensions/emit.ts.
// TS-Go 7.0.2 rewrites the relative .tsx import to .jsx in emitted JavaScript.
#[test]
fn should_rewrite_tsx_import_to_jsx_given_rewrite_option_when_emitting_javascript() {
    // Arrange
    let project = TemporaryProject::new("tsx-import");
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"esnext","module":"preserve","moduleResolution":"bundler","jsx":"preserve","rewriteRelativeImportExtensions":true,"outDir":"dist"},"files":["main.ts","dep.tsx"]}"#,
    );
    project.write(
        "main.ts",
        "import { answer } from \"./dep.tsx\";\nexport const result: number = answer;\n",
    );
    project.write("dep.tsx", "export const answer: number = 42;\n");

    // Act
    let output = project.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    assert_eq!(
        fs::read_to_string(project.path().join("dist/main.js"))
            .expect("the emitted JavaScript can be read"),
        "import { answer } from \"./dep.jsx\";\nexport const result = answer;\n"
    );
}

// Pinned TypeScript fixture: conformance/externalModules/rewriteRelativeImportExtensions/emit.ts.
// TS-Go 7.0.2 rewrites the relative .ts side-effect import to .js in emitted JavaScript.
#[test]
fn should_rewrite_side_effect_typescript_import_given_rewrite_option_when_emitting_javascript() {
    // Arrange
    let project = TemporaryProject::new("side-effect-ts-import");
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"esnext","module":"preserve","moduleResolution":"bundler","rewriteRelativeImportExtensions":true,"outDir":"dist"},"files":["main.ts","dep.ts"]}"#,
    );
    project.write("main.ts", "import \"./dep.ts\";\n");
    project.write("dep.ts", "export const answer: number = 42;\n");

    // Act
    let output = project.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    assert_eq!(
        fs::read_to_string(project.path().join("dist/main.js"))
            .expect("the emitted JavaScript can be read"),
        "import \"./dep.js\";\n"
    );
}

// Pinned TypeScript fixture: conformance/externalModules/rewriteRelativeImportExtensions/emit.ts.
// TS-Go 7.0.2 rewrites the relative .ts star re-export to .js in emitted JavaScript.
#[test]
fn should_rewrite_typescript_star_reexport_given_rewrite_option_when_emitting_javascript() {
    // Arrange
    let project = TemporaryProject::new("ts-star-reexport");
    project.write(
        "tsconfig.json",
        r#"{"compilerOptions":{"target":"esnext","module":"preserve","moduleResolution":"bundler","rewriteRelativeImportExtensions":true,"outDir":"dist"},"files":["main.ts","dep.ts"]}"#,
    );
    project.write("main.ts", "export * from \"./dep.ts\";\n");
    project.write("dep.ts", "export const answer: number = 42;\n");

    // Act
    let output = project.run_cli();
    let diagnostics = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Assert
    assert!(output.status.success(), "{diagnostics}");
    assert_eq!(
        fs::read_to_string(project.path().join("dist/main.js"))
            .expect("the emitted JavaScript can be read"),
        "export * from \"./dep.js\";\n"
    );
}
