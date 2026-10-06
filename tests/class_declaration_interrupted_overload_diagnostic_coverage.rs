use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_ts2391_given_method_overload_followed_by_constructor_when_checking_types() {
    // Pinned fixture: compiler/ClassDeclaration14.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("class-declaration.ts"),
        "class C { foo(); constructor(); }",
    )
    .expect("a TypeScript path has a supported source kind");
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
            .any(|diagnostic| diagnostic.code() == 2391),
        "a method overload followed by a constructor declaration should report TS2391, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts7010_given_class_method_overload_without_return_annotation_when_checking_types()
{
    // Pinned fixture: compiler/ClassDeclaration14.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("class-declaration.ts"),
        "class C { foo(); constructor(); }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(
        CompilerOptions::new(ScriptTarget::Es2015).with_strict_null_checks(false),
    );

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
        "an unannotated class method overload should report TS7010, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts2390_given_constructor_overload_after_method_overload_when_checking_types() {
    // Pinned fixture: compiler/ClassDeclaration14.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("class-declaration.ts"),
        "class C { foo(); constructor(); }",
    )
    .expect("a TypeScript path has a supported source kind");
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
            .any(|diagnostic| diagnostic.code() == 2390),
        "a constructor overload without an implementation should report TS2390, got {:?}",
        result.diagnostics()
    );
}
