use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_ts2511_given_abstract_class_when_instantiating_directly() {
    // Pinned TypeScript 7.0.2 case: conformance/classes/classDeclarations/classAbstractKeyword/classAbstractInstantiations1.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-instantiation.ts"),
        "abstract class A {}\nnew A();",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 2511
                && diagnostic
                    .message()
                    .contains("Cannot create an instance of an abstract class")
        }),
        "instantiating an abstract class should report TS2511, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts2513_given_abstract_method_when_calling_through_super() {
    // Pinned TypeScript 7.0.2 case: conformance/classes/classDeclarations/classAbstractKeyword/classAbstractSuperCalls.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-super-call.ts"),
        "abstract class B { abstract foo(): number; }\nclass C extends B { foo() { return super.foo(); } }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 2513
                && diagnostic.message().contains(
                    "Abstract method 'foo' in class 'B' cannot be accessed via super expression",
                )
        }),
        "calling an abstract method through super should report TS2513, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts1244_given_abstract_method_in_concrete_class_when_checking_types() {
    // Pinned TypeScript 7.0.2 case: conformance/classes/classDeclarations/classAbstractKeyword/classAbstractMethodInNonAbstractClass.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-method-concrete-class.ts"),
        "class A { abstract foo(): void; }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 1244
                && diagnostic
                    .message()
                    .contains("Abstract methods can only appear within an abstract class")
        }),
        "an abstract method in a concrete class should report TS1244, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts1245_given_abstract_method_with_body_when_checking_types() {
    // Pinned TypeScript 7.0.2 case: conformance/classes/classDeclarations/classAbstractKeyword/classAbstractMethodWithImplementation.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-method-body.ts"),
        "abstract class A { abstract foo() {} }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 1245
                && diagnostic.message().contains(
                    "Method 'foo' cannot have an implementation because it is marked abstract",
                )
        }),
        "an abstract method with a body should report TS1245, got {:?}",
        result.diagnostics()
    );
}
