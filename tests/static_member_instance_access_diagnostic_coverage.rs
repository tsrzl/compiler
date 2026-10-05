use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_static_member_access_suggestion_given_instance_property_read_when_checking_types()
{
    // Arrange
    // Pinned fixture: compiler/classStaticPropertyAccess.ts.
    let source = SourceFile::from_path(
        Path::new("static-member.ts"),
        "class Shape { static area: number = 1; } const shape = new Shape(); shape.area;",
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
            .any(|diagnostic| diagnostic.code() == 2576),
        "expected TS2576 with a suggestion to access the static member on its class, got {:?}",
        result.diagnostics()
    );
}
