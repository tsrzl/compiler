use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript 7.0.2 case: conformance/types/typeParameters/typeParameterLists/typeParameterUsedAsConstraint.ts.
// TS-Go accepts a class declaration with a type parameter.
#[test]
fn should_parse_generic_class_declaration_given_type_parameter_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("main.ts"), "class Box<T> {}")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

// Pinned TypeScript 7.0.2 case: conformance/types/typeParameters/typeParameterLists/typeParameterUsedAsConstraint.ts.
// TS-Go accepts an interface declaration with a type parameter.
#[test]
fn should_parse_generic_interface_declaration_given_type_parameter_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("main.ts"), "interface Item<T> {}")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

// Pinned TypeScript 7.0.2 case: conformance/async/es6/asyncAliasReturnType_es6.ts.
// TS-Go accepts a generic type alias.
#[test]
fn should_parse_generic_type_alias_given_type_parameter_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("main.ts"), "type Alias<T> = T;")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

// Pinned TypeScript 7.0.2 case: conformance/types/typeParameters/typeParameterLists/typeParametersAvailableInNestedScope.ts.
// TS-Go accepts a generic arrow function assigned to a variable.
#[test]
fn should_parse_generic_arrow_function_given_type_parameter_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "const identity = <T>(value: T): T => value;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
