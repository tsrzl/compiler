use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_ts2369_given_parameter_property_in_setter_when_checking_types() {
    // Pinned fixture: compiler/MemberAccessorDeclaration15.ts.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("setter-parameter-property.ts"),
        "class C { set Foo(public a: number) { } }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().iter().any(|diagnostic| {
            diagnostic.code() == 2369
                && diagnostic
                    .message()
                    .contains("only allowed in a constructor implementation")
        }),
        "a setter parameter property should report TS2369, got {:?}",
        result.diagnostics()
    );
}
