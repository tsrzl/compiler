use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_boolean_enum_initializer_given_computed_member_when_checking_types() {
    // TypeScript 7.0.2 case: conformance/enums/enumErrors.ts.
    // With target ES2015, the oracle reports TS18033 for a boolean enum initializer.
    // Arrange
    let source = SourceFile::from_path(Path::new("enumErrors.ts"), "enum Flag { On = true }")
        .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 18033),
        "a computed enum member initializer must be number-compatible"
    );
}

#[test]
fn should_report_boxed_number_given_computed_enum_member_when_checking_types() {
    // Pinned TypeScript 7.0.2 case: conformance/enums/enumErrors.ts reports TS18033 for `new Number(30)`.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("enumErrors.ts"),
        "enum Value { Boxed = new Number(30) }",
    )
    .expect("the TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 18033),
        "a boxed Number is not valid as a computed enum member value: {:?}",
        result.diagnostics()
    );
}
