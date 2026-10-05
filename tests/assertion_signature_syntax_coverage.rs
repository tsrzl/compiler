use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_assertion_signature_given_function_declaration_when_building_syntax_tree() {
    // Upstream: conformance/controlFlow/assertionTypePredicates1.ts.
    // With target ES2015, strict, and declaration enabled, TS-Go accepts `asserts value`.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("assertion-signature.ts"),
        "declare function assert(value: unknown): asserts value;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
