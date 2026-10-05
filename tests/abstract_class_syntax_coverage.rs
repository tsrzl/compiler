use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript 7.0.2 case: conformance/classes/classAbstractKeyword/classAbstractSingleLineDecl.ts.
// TS-Go accepts an abstract class declaration.
#[test]
fn should_parse_abstract_class_given_abstract_modifier_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("main.ts"), "abstract class Shape {}")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
    let class = syntax_tree.program().statements()[0]
        .as_class_declaration()
        .expect("the abstract declaration produces a class node");
    assert_eq!(syntax_tree.program().statements().len(), 1);
    assert_eq!(class.name(), "Shape");
}
