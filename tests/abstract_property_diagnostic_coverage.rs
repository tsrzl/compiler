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
