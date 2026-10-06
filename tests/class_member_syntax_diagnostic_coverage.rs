use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_ts1440_given_variable_declaration_in_class_member_when_building_syntax_tree() {
    // Pinned fixture: compiler/ClassDeclaration26.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("invalid-class-member.ts"),
        "class C { public const var export foo = 10; }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 1440
                && diagnostic
                    .message()
                    .contains("Variable declaration not allowed at this location")
        }),
        "a variable declaration in a class member should report TS1440, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts1068_given_var_constructor_member_when_building_syntax_tree() {
    // Pinned fixture: compiler/ClassDeclaration26.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("invalid-class-member.ts"),
        "class C { var constructor() { } }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 1068
                && diagnostic
                    .message()
                    .contains("A constructor, method, accessor, or property was expected")
        }),
        "a var-modified constructor member should report TS1068, got {:?}",
        result.diagnostics()
    );
}
