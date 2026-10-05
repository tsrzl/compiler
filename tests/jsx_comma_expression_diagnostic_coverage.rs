use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_report_comma_operator_in_jsx_expression_given_tsx_source_when_building_syntax_tree() {
    // Arrange
    // Pinned fixture: conformance/jsx/jsxParsingError1.tsx.
    let source = SourceFile::from_path(
        Path::new("view.tsx"),
        "const first = 1; const second = 2; const Div: any = 0; const view = <Div>{first, second}</Div>;",
    )
    .expect("a TSX path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert!(
        syntax_tree
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 18007),
        "expected TS18007 for a comma operator in a JSX expression, got {:?}",
        syntax_tree.diagnostics()
    );
}
