use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_constrained_infer_given_conditional_type_when_building_syntax_tree() {
    // Upstream: conformance/types/conditional/inferTypesWithExtends1.ts.
    // Its target=ES2015, strict, declaration-enabled baseline accepts constrained `infer`.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("constrained-infer.ts"),
        "type Inferred = unknown extends infer U extends string ? U : never;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
