use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_reject_number_assignment_given_resizable_flag_when_checking_types() {
    // TypeScript 7.0.2 case: conformance/es2024/resizableArrayBuffer.ts types `buffer.resizable` as boolean.
    // The pinned TS-Go CLI reports TS2322 when that flag is assigned to number.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("resizableArrayBuffer.ts"),
        "const buffer = new ArrayBuffer(8, { maxByteLength: 16 }); const mustBeNumber: number = buffer.resizable;",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Latest));

    // Act
    let result = compiler.compile(source);

    // Assert
    let diagnostic_codes: Vec<_> = result
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code())
        .collect();
    assert!(
        diagnostic_codes.contains(&2322),
        "buffer.resizable is boolean and is not assignable to number; got diagnostics {diagnostic_codes:?}"
    );
}
