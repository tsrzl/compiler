use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_reject_protected_member_access_through_base_receiver_given_derived_class_method_when_checking_types()
 {
    // Arrange
    // Pinned fixture: conformance/classes/members/accessibility/protectedInstanceMemberAccessibility.ts.
    let source = SourceFile::from_path(
        Path::new("protected-receiver.ts"),
        "class Base { protected value: string = \"\"; } class Derived extends Base { read(base: Base): string { return base.value; } }",
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
            .any(|diagnostic| diagnostic.code() == 2446),
        "expected TS2446 for protected access through a base-class receiver, got {:?}",
        result.diagnostics()
    );
}
