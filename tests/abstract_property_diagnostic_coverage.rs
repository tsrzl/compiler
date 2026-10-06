use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_ts2654_given_concrete_subclass_missing_abstract_members_when_checking_types() {
    // Pinned fixture: compiler/abstractPropertyNegative.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-property.ts"),
        "abstract class Base { abstract value: number; abstract method(): void; } class Derived extends Base {}",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 2654
                && diagnostic
                    .message()
                    .contains("missing implementations for the following members")
        }),
        "a concrete subclass missing abstract members should report TS2654, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts1253_given_abstract_property_in_concrete_class_when_checking_types() {
    // Pinned fixture: compiler/abstractPropertyNegative.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-property.ts"),
        "class C { abstract value: string; }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 1253
                && diagnostic
                    .message()
                    .contains("Abstract properties can only appear within an abstract class")
        }),
        "an abstract property in a concrete class should report TS1253, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts2416_given_incompatible_abstract_property_override_when_checking_types() {
    // Pinned fixture: compiler/abstractPropertyNegative.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-property.ts"),
        "abstract class Base { abstract value: number; } class Derived extends Base { value: string = \"\"; }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 2416 && diagnostic.message().contains("not assignable")
        }),
        "an incompatible abstract property override should report TS2416, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts2676_given_abstract_and_concrete_accessor_pair_when_checking_types() {
    // Pinned fixture: compiler/abstractPropertyNegative.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-property.ts"),
        "abstract class C { abstract get value(): number; set value(value: number) { } }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 2676
                && diagnostic
                    .message()
                    .contains("both be abstract or non-abstract")
        }),
        "an abstract getter paired with a concrete setter should report TS2676, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts2676_given_concrete_getter_and_abstract_setter_when_checking_types() {
    // Pinned fixture: compiler/abstractPropertyNegative.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-property.ts"),
        "abstract class C { get value(): number { return 0; } abstract set value(value: number); }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 2676
                && diagnostic
                    .message()
                    .contains("both be abstract or non-abstract")
        }),
        "a concrete getter paired with an abstract setter should report TS2676, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts2416_given_incompatible_getter_override_for_abstract_getter_when_checking_types()
{
    // Pinned fixture: compiler/abstractPropertyNegative.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-property.ts"),
        "abstract class Base { abstract get value(): number; } class Derived extends Base { get value() { return \"wrong\"; } }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 2416 && diagnostic.message().contains("not assignable")
        }),
        "an incompatible getter overriding an abstract getter should report TS2416, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts2416_given_incompatible_field_override_for_abstract_getter_when_checking_types()
{
    // Pinned fixture: compiler/abstractPropertyNegative.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-property.ts"),
        "abstract class Base { abstract get value(): number; } class Derived extends Base { value = \"wrong\"; }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 2416 && diagnostic.message().contains("not assignable")
        }),
        "an incompatible field overriding an abstract getter should report TS2416, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts1318_given_abstract_getter_with_body_when_checking_types() {
    // Pinned fixture: conformance/classes/classDeclarations/classAbstractKeyword/classAbstractAccessor.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-accessor.ts"),
        "abstract class Base { abstract get value(): number { return 1; } }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 1318
                && diagnostic
                    .message()
                    .contains("An abstract accessor cannot have an implementation")
        }),
        "an abstract getter with a body should report TS1318, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts1318_given_abstract_setter_with_body_when_checking_types() {
    // Pinned fixture: conformance/classes/classDeclarations/classAbstractKeyword/classAbstractAccessor.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-accessor.ts"),
        "abstract class Base { abstract set value(value: number) {} }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 1318
                && diagnostic
                    .message()
                    .contains("An abstract accessor cannot have an implementation")
        }),
        "an abstract setter with a body should report TS1318, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts1243_given_private_abstract_property_when_checking_types() {
    // Pinned fixture: conformance/classes/classDeclarations/classAbstractKeyword/classAbstractProperties.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-property.ts"),
        "abstract class Base { private abstract value: number; }",
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
            .any(|diagnostic| diagnostic.code() == 1243),
        "a private abstract property should report TS1243, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts1243_given_private_abstract_method_when_checking_types() {
    // Pinned fixture: conformance/classes/classDeclarations/classAbstractKeyword/classAbstractProperties.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("abstract-method.ts"),
        "abstract class Base { private abstract method(): void; }",
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
            .any(|diagnostic| diagnostic.code() == 1243),
        "a private abstract method should report TS1243, got {:?}",
        result.diagnostics()
    );
}
