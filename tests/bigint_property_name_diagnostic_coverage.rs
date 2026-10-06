use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_ts1539_given_bigint_literal_object_property_when_checking_types() {
    // Pinned fixture: compiler/bigintPropertyName.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("bigint-property.ts"),
        "const invalid = { 1n: 123 };",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Latest));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 1539),
        "a BigInt literal used as an object property should report TS1539, got {:?}",
        result.diagnostics()
    );
}
