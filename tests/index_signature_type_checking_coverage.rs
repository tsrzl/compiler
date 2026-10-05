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

#[test]
fn should_report_numeric_property_mismatch_given_numeric_index_signature_when_checking_types() {
    // TypeScript 7.0.2 case: conformance/types/objectTypeLiteral/indexSignatures/numericIndexerConstrainsPropertyDeclarations.ts.
    // With target ES2015 and strict=false, the incompatible numeric property reports TS2411.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("numericIndexerConstrainsPropertyDeclarations.ts"),
        "interface NumericMap { [key: number]: string; 1: number; }",
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
        "a declared numeric property must satisfy its numeric index signature"
    );
}

#[test]
fn should_read_numeric_index_signature_value_given_number_key_when_checking_types() {
    // TypeScript 7.0.2 case: conformance/types/objectTypeLiteral/indexSignatures/numericIndexingResults.ts.
    // With strict=false, reading through a numeric index signature produces its declared value type.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("numericIndexingResults.ts"),
        "interface NumericMap { [key: number]: string; } declare const values: NumericMap; const value: string = values[1];",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_strict_null_checks(false),
    );

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}
