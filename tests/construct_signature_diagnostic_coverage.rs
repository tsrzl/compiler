use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_ts2369_given_constructor_signature_parameter_property_when_checking_types() {
    // Pinned fixture: compiler/ParameterList13.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("constructor-signature.ts"),
        "interface I { new (public x: number): I; }",
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
            .any(|diagnostic| diagnostic.code() == 2369),
        "a construct-signature parameter property should report TS2369, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts7013_given_construct_signature_without_return_type_when_checking_types() {
    // Pinned fixture: compiler/ParameterList13.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("construct-signature-return.ts"),
        "interface I { new (); }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 7013
                && diagnostic
                    .message()
                    .contains("lacks return-type annotation")
        }),
        "a construct signature without a return type should report TS7013, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts7006_given_untyped_construct_signature_parameter_when_checking_types() {
    // Pinned fixture: compiler/ParameterList13.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("construct-signature-parameter.ts"),
        "interface I { new (x): I; }",
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
            .any(|diagnostic| diagnostic.code() == 7006),
        "an untyped construct-signature parameter should report TS7006, got {:?}",
        result.diagnostics()
    );
}
