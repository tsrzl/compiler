use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_non_null_assertion_given_variable_expression_when_building_syntax_tree() {
    // Arrange
    // Pinned behavior fixture: compiler/narrowingWithNonNullExpression.ts.
    let source = SourceFile::from_path(Path::new("non-null.ts"), "const asserted = value!;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
    let initializer = syntax_tree.program().statements()[0]
        .as_variable_declaration()
        .and_then(|declaration| declaration.initializer())
        .expect("the variable has a non-null asserted initializer");
    let span = initializer.span();
    let start = span.start().get();
    let end = start + span.length();
    assert_eq!(
        &syntax_tree.source_file().text().as_str()[start..end],
        "value!"
    );
}
