use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_insert_return_terminator_given_line_break_before_expression_when_emitting_javascript() {
    // Arrange
    // Pinned fixture: conformance/statements/returnStatements/returnStatementNoAsiAfterTransform.ts.
    let source = SourceFile::from_path(
        Path::new("return-asi.ts"),
        "function getValue() {\n    return\n    42;\n}",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015)).compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(
        result.emitted_files()[0]
            .text()
            .contains("return;\n    42;"),
        "a line terminator after return must leave the following expression separate: {}",
        result.emitted_files()[0].text()
    );
}
