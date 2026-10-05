use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_required_return_value_given_non_void_function_without_return_when_checking_types()
{
    // Arrange
    // Pinned TS-Go 7.0.2 reports TS2355; see compiler/missingReturnStatement.errors.txt.
    let source = SourceFile::from_path(
        Path::new("missingReturnStatement.ts"),
        "function mustReturn(): number {}",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2355),
        "expected TS2355 for a non-void function with no return value, got {:?}",
        result.diagnostics()
    );
}
