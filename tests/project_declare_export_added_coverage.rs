use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

// Pinned TypeScript 7.0.2 case: projects/DeclareExportAdded/consumer.ts.
#[test]
fn should_preserve_triple_slash_reference_given_declaration_path_when_emitting_javascript() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("consumer.ts"),
        "///<reference path=\"ref.d.ts\" />\n\n// in the generated code a 'this' is added before this call\nM1.f1();",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .emitted_files()
            .first()
            .expect("JavaScript is emitted for the source file")
            .text()
            .contains("///<reference path=\"ref.d.ts\" />"),
        "the emitted JavaScript should preserve the triple-slash reference directive"
    );
}
