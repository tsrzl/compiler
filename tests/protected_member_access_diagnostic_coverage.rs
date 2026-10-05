use std::path::Path;

use tsrzl::compiler::{Compiler, CompilerOptions, ScriptTarget};
use tsrzl::source_file::SourceFile;

#[test]
fn should_reject_protected_property_access_given_external_instance_when_checking_types() {
    // Arrange
    // Pinned fixture: conformance/classes/members/accessibility/classPropertyAsProtected.ts.
    let source = SourceFile::from_path(
        Path::new("protected-member.ts"),
        "class Shape { protected width: number = 1; } const shape = new Shape(); shape.width;",
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
            .any(|diagnostic| diagnostic.code() == 2445),
        "expected TS2445 for external access to a protected property, got {:?}",
        result.diagnostics()
    );
}
