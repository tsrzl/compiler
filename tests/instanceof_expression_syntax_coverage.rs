use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_instanceof_operator_given_return_expression_when_building_syntax_tree() {
    // Arrange
    // Pinned fixture: conformance/controlFlow/controlFlowInstanceOfGuardPrimitives.ts.
    let source = SourceFile::from_path(
        Path::new("instanceof.ts"),
        "function isX(value: X | number) { return value instanceof X; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
    let function = syntax_tree.program().statements()[0]
        .as_function_declaration()
        .expect("the instanceof expression belongs to a function");
    let expression =
        function.return_expressions()[0].expect("the function returns the instanceof expression");
    let span = expression.span();
    let start = span.start().get();
    let end = start + span.length();
    assert_eq!(
        &syntax_tree.source_file().text().as_str()[start..end],
        "value instanceof X"
    );
}
