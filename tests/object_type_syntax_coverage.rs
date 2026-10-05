use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript 7.0.2 case: conformance/types/objectTypeLiteral/methodSignatures/functionLiterals.ts.
// TS-Go accepts a named method signature in an object type literal.
#[test]
fn should_parse_method_signature_in_object_type_given_typed_parameter_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "type Service = { read(path: string): string; };",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

// Pinned TypeScript 7.0.2 case: conformance/types/objectTypeLiteral/methodSignatures/objectTypesWithOptionalProperties.ts.
// TS-Go accepts an optional property signature in an object type literal.
#[test]
fn should_parse_optional_property_in_object_type_given_question_mark_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(Path::new("main.ts"), "type Options = { name?: string; };")
        .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

// Pinned TypeScript 7.0.2 case: conformance/types/objectTypeLiteral/indexSignatures/stringIndexingResults.ts.
// TS-Go accepts a string index signature in an object type literal.
#[test]
fn should_parse_string_index_signature_in_object_type_given_string_key_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "type Dictionary = { [key: string]: number; };",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

// Pinned TypeScript 7.0.2 case: conformance/controlFlow/controlFlowAliasing.ts.
// TS-Go accepts a readonly property in an object type literal.
#[test]
fn should_parse_readonly_property_in_object_type_given_readonly_modifier_when_building_syntax_tree()
{
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "type ReadonlyValue = { readonly value: number; };",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
