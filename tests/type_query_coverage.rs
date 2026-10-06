use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript 7.0.2 case: conformance/types/specifyingTypes/typeQueries/circularTypeofWithVarOrFunc.ts.
// TS-Go accepts typeof a value name in a type alias.
#[test]
fn should_preserve_value_type_query_given_type_alias_when_emitting_declarations() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("main.ts"),
        "const value = 1;\ntype Value = typeof value;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015)
            .with_declaration(true)
            .with_emit_declaration_only(true),
    )
    .compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    let declaration = result.emitted_files()[0].text();
    assert!(
        declaration.contains("type Value = typeof value;"),
        "the declaration should preserve the value type query, got {declaration:?}"
    );
}

#[test]
fn should_accept_class_type_query_given_class_name_when_checking_types() {
    // Pinned fixture: conformance/types/specifyingTypes/typeQueries/typeQueryOnClass.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("class-query.ts"),
        "class Box {}\nlet constructorType: typeof Box;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_accept_instance_type_query_given_instance_name_when_checking_types() {
    // Pinned fixture: conformance/types/specifyingTypes/typeQueries/typeQueryOnClass.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("instance-query.ts"),
        "class Box {}\nlet instance: Box;\nlet instanceType: typeof instance;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_accept_qualified_type_query_given_class_member_when_checking_types() {
    // Pinned fixture: conformance/types/specifyingTypes/typeQueries/typeQueryWithReservedWords.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("qualified-query.ts"),
        "class Controller { create(): void {} }\ntype Method = typeof Controller.prototype.create;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}

#[test]
fn should_parse_this_member_type_query_given_class_property_when_building_syntax_tree() {
    // Pinned fixture: conformance/types/specifyingTypes/typeQueries/typeofThis.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("this-query.ts"),
        "class Box { value: typeof this.value; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_report_circular_type_query_given_self_referential_variable_annotation_when_checking_types()
 {
    // Pinned fixture: conformance/types/specifyingTypes/typeQueries/recursiveTypesWithTypeof.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("recursive-query.ts"),
        "var node: typeof node;\nvar node: any;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2502),
        "the circular type query should report TS2502, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_accept_enum_member_type_query_given_enum_member_when_checking_types() {
    // Pinned fixture: conformance/types/specifyingTypes/typeQueries/typeofANonExportedType.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("enum-query.ts"),
        "enum Color { Red }\ntype RedValue = typeof Color.Red;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}
