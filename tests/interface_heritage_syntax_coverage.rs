use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript 7.0.2 case: conformance/externalModules/typeOnly/implementsClause.ts.
// TS-Go accepts a class implementing an interface through a qualified name.
#[test]
fn should_parse_qualified_implements_clause_given_dotted_interface_name_when_building_syntax_tree()
{
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "class C implements types.Component {}",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

// Pinned TypeScript 7.0.2 case: conformance/externalModules/typeOnly/extendsClause.ts.
// TS-Go accepts a qualified type in an interface heritage clause.
#[test]
fn should_parse_qualified_interface_heritage_given_dotted_base_name_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("main.ts"), "interface S extends types.C {}")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

// Pinned TypeScript 7.0.2 case: conformance/interfaces/interfaceDeclarations/interfaceWithMultipleBaseTypes.ts.
// TS-Go accepts an interface that extends two base interfaces.
#[test]
fn should_parse_multiple_interface_heritage_types_given_comma_separated_bases_when_building_syntax_tree()
 {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "interface BaseA {}\ninterface BaseB {}\ninterface Derived extends BaseA, BaseB {}",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
