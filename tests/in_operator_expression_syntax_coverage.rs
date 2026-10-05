use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_in_operator_given_return_expression_when_building_syntax_tree() {
    // Arrange
    // Pinned fixture: conformance/expressions/typeGuards/typeGuardOfFromPropNameInUnionType.ts.
    let source = SourceFile::from_path(
        Path::new("in-operator.ts"),
        "function hasA(value: any) { return \"a\" in value; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
    let function = syntax_tree.program().statements()[0]
        .as_function_declaration()
        .expect("the in-operator expression belongs to a function");
    let expression =
        function.return_expressions()[0].expect("the function returns the in-operator expression");
    let span = expression.span();
    let start = span.start().get();
    let end = start + span.length();
    assert_eq!(
        &syntax_tree.source_file().text().as_str()[start..end],
        "\"a\" in value"
    );
}
