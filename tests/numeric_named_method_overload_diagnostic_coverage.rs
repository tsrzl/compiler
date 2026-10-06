use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_ts7010_given_numeric_named_method_without_return_annotation_when_checking_types() {
    // Pinned fixture: compiler/ClassDeclaration21.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("numeric-method-overload.ts"),
        "class C { 0(); 1() { } }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 7010
                && diagnostic
                    .message()
                    .contains("lacks return-type annotation")
        }),
        "a numeric-named overload without a return annotation should report TS7010, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts2389_given_numeric_method_implementation_name_mismatch_when_checking_types() {
    // Pinned fixture: compiler/ClassDeclaration21.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("numeric-method-overload.ts"),
        "class C { 0(); 1() { } }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 2389
                && diagnostic
                    .message()
                    .contains("implementation name must be '0'")
        }),
        "a numeric-named overload followed by another method should report TS2389, got {:?}",
        result.diagnostics()
    );
}
