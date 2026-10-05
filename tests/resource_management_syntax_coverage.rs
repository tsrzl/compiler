use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_await_using_declaration_given_null_initializer_when_building_tree() {
    // TypeScript 7.0.2 case: conformance/statements/VariableStatements/usingDeclarations/awaitUsingDeclarations.1.ts.
    // With target ESNext and module ESNext, the fixture parses without diagnostics.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("await-using.ts"),
        "await using resource = null; export {};",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
    assert!(
        syntax_tree.program().statements()[0]
            .as_variable_declaration()
            .is_some(),
        "the await using construct is a variable declaration"
    );
}
