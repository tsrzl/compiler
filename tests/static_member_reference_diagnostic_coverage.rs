use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_report_static_member_suggestion_given_unqualified_static_member_reference_when_checking_types()
 {
    // Arrange
    // Pinned fixture: compiler/accessInstanceMemberFromStaticMethod01.ts.
    let source = SourceFile::from_path(
        Path::new("static-member.ts"),
        "class C { static foo: string; bar() { let k = foo; } }",
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
            .any(|diagnostic| diagnostic.code() == 2662),
        "expected TS2662 with a suggestion for the static member, got {:?}",
        result.diagnostics()
    );
}
