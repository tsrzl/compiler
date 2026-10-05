use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_narrow_union_given_in_operator_true_branch_when_checking_types() {
    // Arrange
    // Pinned fixture: conformance/expressions/typeGuards/typeGuardOfFromPropNameInUnionType.ts.
    let source = SourceFile::from_path(
        Path::new("in-narrowing.ts"),
        "class A { a: string = \"\"; } class B { b: number = 0; } function get(value: A | B) { if (\"a\" in value) { const result: string = value.a; } }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().is_empty(),
        "the in-operator true branch should narrow to class A, got {:?}",
        result.diagnostics()
    );
}
