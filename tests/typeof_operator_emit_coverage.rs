use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

// Pinned TypeScript fixture: conformance/expressions/unaryOperators/typeofOperator/typeofOperatorWithStringType.ts.
#[test]
fn should_emit_typeof_expression_given_identifier_operand_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("typeof.ts"),
        "const value = \"text\";\nconst kind = typeof value;\n",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "const value = \"text\";\nconst kind = typeof value;\n"
    );
}
