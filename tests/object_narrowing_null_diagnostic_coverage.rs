use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_possible_null_given_nullable_object_property_read_when_checking_types() {
    // Arrange
    // Pinned fixture: conformance/types/nonPrimitive/nonPrimitiveStrictNull.ts,
    // target=es2015, strictNullChecks=true. This isolates its TS18047 property read.
    let source = SourceFile::from_path(
        Path::new("nonPrimitiveStrictNull.ts"),
        "let d: object | null = null; d.toString();",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_strict_null_checks(true),
    );

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 18047),
        "expected TS18047 for reading a nullable object's property, got {:?}",
        result.diagnostics()
    );
}
