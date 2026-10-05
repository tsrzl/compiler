use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript 7.0.2 case: conformance/types/typeParameters/typeParameterLists/varianceAnnotations.ts.
// TS-Go accepts an `out` variance annotation on a generic type parameter.
#[test]
fn should_parse_covariant_type_parameter_given_out_variance_annotation_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "type Covariant<out T> = { value: T };",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
