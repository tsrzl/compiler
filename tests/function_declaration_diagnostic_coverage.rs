use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_ts2391_given_function_declaration_without_implementation_when_checking_types() {
    // Pinned fixture: compiler/FunctionDeclaration3.ts.
    // Arrange
    let source = SourceFile::from_path(Path::new("function-declaration.ts"), "function foo();")
        .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 2391 && diagnostic.message().contains("implementation is missing")
        }),
        "a function declaration without an implementation should report TS2391, got {:?}",
        result.diagnostics()
    );
}

#[test]
fn should_report_ts7010_given_function_declaration_without_return_annotation_when_checking_types() {
    // Pinned fixture: compiler/FunctionDeclaration3.ts.
    // Arrange
    let source = SourceFile::from_path(Path::new("function-declaration.ts"), "function foo();")
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
        "an unannotated function declaration should report TS7010, got {:?}",
        result.diagnostics()
    );
}
