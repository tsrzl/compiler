use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript 7.0.2 case: conformance/classes/classDeclarations/classAbstractKeyword/classAbstractGeneric.ts.
// TS-Go accepts an abstract method signature in an abstract class.
#[test]
fn should_parse_abstract_method_signature_given_abstract_class_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "abstract class Shape { abstract area(): number; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
    let class = syntax_tree.program().statements()[0]
        .as_class_declaration()
        .expect("the abstract method belongs to a class node");
    assert_eq!(class.members().len(), 1);
    assert_eq!(
        class.members()[0]
            .as_method()
            .expect("the abstract signature produces a method node")
            .name(),
        "area"
    );
}
