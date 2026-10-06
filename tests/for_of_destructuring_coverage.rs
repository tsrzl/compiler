use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;
use tsrzl::syntax::SyntaxTree;

#[test]
fn should_parse_array_binding_pattern_given_for_of_statement_when_building_syntax_tree() {
    // Pinned fixture: conformance/es6/for-ofStatements/for-of38.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("for-of.ts"),
        "var entries = [[\"\", true]];\nfor (var [key, value] of entries) { key; value; }",
    )
    .expect("a TypeScript path has a supported source kind");

    // Act
    let syntax_tree = SyntaxTree::parse(source);

    // Assert
    assert_eq!(syntax_tree.diagnostics(), []);
}

#[test]
fn should_preserve_array_binding_pattern_given_for_of_statement_when_emitting_javascript() {
    // Pinned fixture: conformance/es6/for-ofStatements/for-of38.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("for-of.ts"),
        "var entries = [[\"\", true]];\nfor (var [key, value] of entries) { key; value; }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
    assert!(
        result.emitted_files()[0]
            .text()
            .contains("for (var [key, value] of entries)"),
        "the emitted loop should preserve its binding pattern, got {:?}",
        result.emitted_files()[0].text()
    );
}
