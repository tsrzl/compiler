use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_reject_symbol_given_bigint_constructor_argument_when_checking_types() {
    // TypeScript 7.0.2 case: conformance/es2020/constructBigint.ts.
    // With target ESNext and lib ESNext, BigInt rejects a symbol argument with TS2345.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("constructBigint.ts"),
        "declare const invalid: symbol; BigInt(invalid);",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Latest));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2345),
        "the BigInt constructor does not accept symbol values"
    );
}

#[test]
fn should_accept_string_given_bigint_constructor_argument_when_checking_types() {
    // TypeScript 7.0.2 case: conformance/es2020/constructBigint.ts.
    // With target ESNext and lib ESNext, BigInt accepts a string argument.
    // Arrange
    let source = SourceFile::from_path(Path::new("constructBigint.ts"), "BigInt(\"0\");")
        .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Latest));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}
