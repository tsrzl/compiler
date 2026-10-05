use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript 7.0.2 project case: projects/moduleMergeOrder/a.ts.
// Namespace declarations parse successfully in both module input orders.
#[test]
fn should_parse_namespace_declaration_given_namespace_block_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("a.ts"),
        "namespace Test { class A { value: string; } export class B {} }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
