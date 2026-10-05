use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

// Pinned TypeScript project case: projects/NestedLocalModule-SimpleCase/test1.ts.
// TS-Go parses this nested import-equals form, then reports TS1147 semantically.
#[test]
fn should_parse_import_equals_given_namespace_declaration_when_building_syntax_tree() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("test1.ts"),
        "namespace myModule { import foo = require(\"./test2\"); }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

// Pinned TypeScript project case: projects/NestedLocalModule-WithRecursiveTypecheck/test1.ts.
// Direct TS-Go 7.0.2 reports TS1147 for the import-equals inside this namespace.
#[test]
fn should_report_invalid_module_reference_given_import_equals_inside_namespace_when_compiling() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("test1.ts"),
        "namespace myModule {\n import foo = require(\"test2\");\n var z = foo.Yo.y();\n}\nexport var x = 0;",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 1147),
        "expected TS1147 for a module reference inside a namespace, got {:?}",
        result.diagnostics()
    );
}
