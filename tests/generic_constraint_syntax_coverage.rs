use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript 7.0.2 case: conformance/types/typeParameters/typeParameterLists/typeParameterUsedAsConstraint.ts.
// TS-Go accepts a constraint on a generic function's type parameter.
#[test]
fn should_parse_type_parameter_constraint_given_generic_function_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "function identity<T extends string>(value: T): T { return value; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
