use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript 7.0.2 case: conformance/types/specifyingTypes/typeLiterals/functionLiteral.ts.
// TS-Go accepts a function type alias with a typed parameter and return type.
#[test]
fn should_parse_function_type_alias_given_typed_parameter_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "type Mapper = (value: string) => number;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

// Pinned TypeScript 7.0.2 case: conformance/types/specifyingTypes/typeLiterals/functionLiteral.ts.
// TS-Go accepts a call signature inside an object type literal.
#[test]
fn should_parse_call_signature_type_literal_given_typed_parameter_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "type Mapper = { (value: string): number; };",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

// Pinned TypeScript 7.0.2 case: conformance/types/specifyingTypes/typeLiterals/functionLiteral.ts.
// TS-Go accepts a construct signature in a type alias.
#[test]
fn should_parse_construct_signature_given_typed_parameter_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "type Factory = new (value: number) => Date;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

// Pinned TypeScript 7.0.2 case: conformance/types/specifyingTypes/typeLiterals/functionLiteral.ts.
// TS-Go accepts a generic call signature in an object type literal.
#[test]
fn should_parse_generic_call_signature_given_type_parameter_when_building_syntax_tree() {
    // Arrange
    let source =
        SourceFile::from_path(Path::new("main.ts"), "type Mapper = { <T>(value: T): T; };")
            .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
