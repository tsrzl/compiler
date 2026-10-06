use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_abstract_property_member_given_abstract_modifier_when_building_syntax_tree() {
    // Pinned TypeScript 7.0.2 case: compiler/abstractPropertyNegative.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-property.ts"),
        "class Shape { abstract value: number; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
    let class = syntax_tree.program().statements()[0]
        .as_class_declaration()
        .expect("the abstract property belongs to a class node");
    assert_eq!(class.members().len(), 1);
    assert_eq!(
        class.members()[0]
            .as_property()
            .expect("the abstract signature produces a property node")
            .name(),
        "value"
    );
}

#[test]
fn should_parse_abstract_getter_signature_given_abstract_class_when_building_syntax_tree() {
    // Pinned TypeScript 7.0.2 case: conformance/classes/classDeclarations/classAbstractKeyword/classAbstractAccessor.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-accessor.ts"),
        "abstract class Shape { abstract get area(): number; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
    let class = syntax_tree.program().statements()[0]
        .as_class_declaration()
        .expect("the abstract getter belongs to a class node");
    assert_eq!(class.members().len(), 1);
}

#[test]
fn should_parse_abstract_setter_signature_given_abstract_class_when_building_syntax_tree() {
    // Pinned TypeScript 7.0.2 case: conformance/classes/classDeclarations/classAbstractKeyword/classAbstractAccessor.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-accessor.ts"),
        "abstract class Shape { abstract set area(value: number); }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
    let class = syntax_tree.program().statements()[0]
        .as_class_declaration()
        .expect("the abstract setter belongs to a class node");
    assert_eq!(class.members().len(), 1);
}
