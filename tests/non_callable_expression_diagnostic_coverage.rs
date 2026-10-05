use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_call_on_non_callable_value_given_number_expression_when_checking_types() {
    // Arrange
    // Pinned TS-Go 7.0.2 reports TS2349; see compiler/callOnInstance.errors.txt.
    let source = SourceFile::from_path(
        Path::new("callOnInstance.ts"),
        "const value: number = 1; value();",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2349),
        "expected TS2349 when calling a number value, got {:?}",
        result.diagnostics()
    );
}
