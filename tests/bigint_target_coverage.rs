use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_bigint_exponentiation_given_es2015_target_when_checking_types() {
    // TypeScript 7.0.2 case: compiler/bigIntWithTargetLessThanES2016.ts.
    // The ES2015 baseline reports TS2791 for BigInt exponentiation.
    // Arrange
    let source = SourceFile::from_path(Path::new("bigint.ts"), "BigInt(1) ** BigInt(1);")
        .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2791),
        "BigInt exponentiation requires an ES2016 or newer target"
    );
}
