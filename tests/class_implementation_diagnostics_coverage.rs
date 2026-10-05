use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

// Pinned TypeScript 7.0.2 case: compiler/ClassDeclaration10.ts.
#[test]
fn should_report_missing_constructor_implementation_given_overload_only_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("ClassDeclaration10.ts"),
        "class C { constructor(); }",
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
            .any(|diagnostic| diagnostic.code() == 2390)
    );
}

#[test]
fn should_report_missing_method_implementation_given_overload_only_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("ClassDeclaration10.ts"), "class C { foo(); }")
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
            .any(|diagnostic| diagnostic.code() == 2391)
    );
}

// Pinned TypeScript 7.0.2 case: compiler/ClassDeclaration13.ts.
#[test]
fn should_report_mismatched_overload_implementation_given_different_method_name_when_checking_types()
 {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("ClassDeclaration13.ts"),
        "class C { foo(); bar() {} }",
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
            .any(|diagnostic| diagnostic.code() == 2389)
    );
}

// Pinned TypeScript 7.0.2 case: compiler/ClassDeclaration24.ts.
#[test]
fn should_reject_reserved_type_keyword_given_class_name_when_checking_types() {
    // Arrange
    let source = SourceFile::from_path(Path::new("ClassDeclaration24.ts"), "class any {}")
        .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2414)
    );
}
