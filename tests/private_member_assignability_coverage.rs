use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

// TypeScript 7.0.2 oracle: conformance/types/typeRelationships/assignmentCompatibility/assignmentCompatWithObjectMembersAccessibility.ts.
// Its target=es2015, strict=false baseline reports TS2322 when a private member is assigned to a public shape.
#[test]
fn should_reject_private_member_as_public_structural_assignment_given_class_instance_when_checking_types()
 {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("assignmentCompatWithObjectMembersAccessibility.ts"),
        "class PublicShape { foo: string; } class PrivateShape { private foo: string; } const hidden: PrivateShape = new PrivateShape(); const visible: PublicShape = hidden;",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code() == 2322),
        "a private class member must not satisfy a public structural member"
    );
}
