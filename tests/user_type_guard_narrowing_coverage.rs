use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_accept_derived_member_access_given_true_branch_of_user_type_guard_when_checking_types() {
    // Arrange
    // Pinned fixture: conformance/expressions/typeGuards/typeGuardFunction.ts.
    let source = SourceFile::from_path(
        Path::new("type-guard.ts"),
        "class Base { base: number = 0; } class Derived extends Base { derived: number = 1; } function isDerived(value: Base): value is Derived { return true; } function use(value: Base) { if (isDerived(value)) { const result: number = value.derived; } }",
    )
    .expect("a TypeScript path has a supported source kind");
    let compiler = Compiler::with_options(CompilerOptions::new(ScriptTarget::Es2015));

    // Act
    let result = compiler.compile(source);

    // Assert
    assert!(
        result.diagnostics().is_empty(),
        "a user type guard should narrow the base value to Derived, got {:?}",
        result.diagnostics()
    );
}
