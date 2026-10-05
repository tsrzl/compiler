use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_erase_non_null_assertion_given_nullable_variable_when_emitting_javascript() {
    // Arrange
    // Pinned behavior fixture: compiler/narrowingWithNonNullExpression.ts.
    let source = SourceFile::from_path(
        Path::new("non-null.ts"),
        "let value: string | undefined = \"text\"; const asserted: string = value!;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert_eq!(
        result.emitted_files()[0].text(),
        "\"use strict\";\nlet value = \"text\";\nconst asserted = value;\n"
    );
}
