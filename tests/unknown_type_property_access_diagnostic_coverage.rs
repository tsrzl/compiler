use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_unknown_type_property_access_given_unchecked_value_when_checking_types() {
    // Arrange
    // Pinned fixture: conformance/types/unknown/unknownType1.ts, target=es2015, strict=true.
    // This reduced unknown-property read also reports TS18046 without strict-only options.
    let source = SourceFile::from_path(
        Path::new("unknownType1.ts"),
        "function f10(x: unknown) { x.foo; }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 18046),
        "expected TS18046 for property access on an unknown value, got {:?}",
        result.diagnostics()
    );
}
