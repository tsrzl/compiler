use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_unassigned_variable_read_given_declaration_without_initializer_when_checking_types()
 {
    // TypeScript 7.0.2 case: conformance/types/stringLiteral/stringLiteralMatchedInSwitch01.ts.
    // The default target configuration reports TS2454 for reading `foo` before assignment.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("stringLiteralMatchedInSwitch01.ts"),
        "var value: string; value;",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2454),
        "reading a variable without an initializer must report TS2454"
    );
}
