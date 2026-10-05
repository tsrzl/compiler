use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_indexed_property_type_mismatch_given_string_index_signature_when_checking_types() {
    // TypeScript 7.0.2 case: conformance/types/objectTypeLiteral/indexSignatures/stringIndexerConstrainsPropertyDeclarations.ts.
    // With target ES2015 and strict=false, the incompatible property reports TS2411.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("stringIndexerConstrainsPropertyDeclarations.ts"),
        "interface StringMap { [key: string]: string; count: number; }",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_strict_null_checks(false),
    );

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2411),
        "a declared property must satisfy its string index signature"
    );
}
