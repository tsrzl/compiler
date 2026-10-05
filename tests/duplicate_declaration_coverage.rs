use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_duplicate_block_scoped_bindings_given_same_scope_when_checking_types() {
    // TypeScript 7.0.2 case: compiler/letDeclarations-scopes-duplicates.ts (target ES6).
    // The reference reports TS2451 for both declarations of the same block-scoped name.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("letDeclarations-scopes-duplicates.ts"),
        "let value = 0; let value = 0;",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(
        result
            .diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.code() == 2451)
            .count(),
        2,
        "each declaration in a same-scope duplicate let binding must report TS2451"
    );
}
