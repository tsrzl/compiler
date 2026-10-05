use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

struct TemporaryProject {
    root: PathBuf,
}

impl TemporaryProject {
    fn new(name: &str, source: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "tsrzl-project-reference-{}-{name}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("the temporary project directory can be created");
        fs::write(root.join("main.ts"), source).expect("the TypeScript source can be written");
        Self { root }
    }

    fn compile(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tsrzl"))
            .arg(self.root.join("main.ts"))
            .output()
            .expect("the compiler CLI can be started")
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

// Pinned TypeScript input: projects/InvalidReferences/main.ts.
// Upstream scenario: project/visibilityOfTypeUsedAcrossModules2.json.
#[test]
fn should_report_source_reference_to_itself_given_triple_slash_directive_when_running_compiler_cli()
{
    // Arrange
    let project = TemporaryProject::new("self-reference", "/// <reference path=\"main.ts\" />");

    // Act
    let output = project.compile();

    // Assert
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    assert!(diagnostics.contains("TS1006:"), "{diagnostics}");
}

// Pinned TypeScript input: projects/InvalidReferences/main.ts.
// Upstream scenario: project/visibilityOfTypeUsedAcrossModules2.json.
#[test]
fn should_report_missing_source_references_given_unresolved_triple_slash_paths_when_running_compiler_cli()
 {
    // Arrange
    let project = TemporaryProject::new(
        "missing-references",
        "/// <reference path=\"nonExistingFile1.ts\" />\n/// <reference path=\"nonExistingFile2.ts\" />",
    );

    // Act
    let output = project.compile();

    // Assert
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    assert_eq!(diagnostics.matches("TS6053:").count(), 2, "{diagnostics}");
}
