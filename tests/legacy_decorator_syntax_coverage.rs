use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript 7.0.2 case: conformance/decorators/class/constructableDecoratorOnClass01.ts.
// TS-Go accepts the legacy decorator syntax and separately reports TS1238 for its signature.
#[test]
fn should_parse_legacy_class_decorator_given_decorated_class_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("constructableDecoratorOnClass01.ts"),
        "class CtorDtor {}\n@CtorDtor class C {}\n",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
