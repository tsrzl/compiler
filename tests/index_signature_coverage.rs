use std::path::Path;

use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript input: conformance/additionalChecks/noPropertyAccessFromIndexSignature1.ts.
#[test]
fn should_parse_string_index_signature_given_interface_member_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("input.ts"),
        "interface Values { [key: string]: string; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

// Pinned TypeScript 7.0.2 input: conformance/types/objectTypeLiteral/indexSignatures/numericIndexingResults.ts.
// TS-Go accepts a numeric index signature as an interface member.
#[test]
fn should_parse_numeric_index_signature_given_interface_member_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("input.ts"),
        "interface Values { [key: number]: string; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

// Pinned TypeScript 7.0.2 input: conformance/types/objectTypeLiteral/indexSignatures/numericIndexingResults.ts.
// TS-Go accepts a numeric index signature in an object type literal.
#[test]
fn should_parse_numeric_index_signature_in_object_type_given_numeric_key_when_building_syntax_tree()
{
    // Arrange
    let source = SourceFile::from_path(
        Path::new("input.ts"),
        "type Values = { [key: number]: string; };",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

// Pinned TypeScript 7.0.2 input: conformance/types/objectTypeLiteral/indexSignatures/numericIndexingResults.ts.
// TS-Go accepts a numeric index signature in a class.
#[test]
fn should_parse_numeric_index_signature_given_class_member_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("input.ts"),
        "class Values { [key: number]: string; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}
