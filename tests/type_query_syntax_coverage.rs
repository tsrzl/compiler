use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

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
