use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_type_predicate_return_type_given_function_declaration_when_building_syntax_tree() {
    // Arrange
    // Pinned fixture: conformance/expressions/typeGuards/typeGuardFunction.ts.
    let source = SourceFile::from_path(
        Path::new("type-guard.ts"),
        "function isDerived(value: Base): value is Derived { return true; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
    let function = syntax_tree.program().statements()[0]
        .as_function_declaration()
        .expect("the type predicate belongs to a function declaration");
    let predicate = function
        .return_type()
        .expect("the function has a type-predicate return annotation")
        .span();
    let start = predicate.start().get();
    let end = start + predicate.length();
    assert_eq!(
        &syntax_tree.source_file().text().as_str()[start..end],
        "value is Derived"
    );
}
