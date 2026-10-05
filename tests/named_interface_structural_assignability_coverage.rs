use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

// TypeScript 7.0.2 oracle: conformance/types/typeRelationships/assignmentCompatibility/assignmentCompatWithObjectMembers.ts.
// The interface-to-interface assignments in the fixture are accepted; pinned TS-Go also accepts this distilled case.
#[test]
fn should_assign_structurally_compatible_named_interfaces_given_variable_initializer_when_checking_types()
 {
    // Arrange
    let source = SourceFile::from_path(
        Path::new("assignmentCompatWithObjectMembers.ts"),
        "interface Source { foo: string; } interface Target { foo: string; } const source: Source = { foo: \"value\" }; const target: Target = source;",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert!(
        result.diagnostics().is_empty(),
        "structurally compatible named interfaces should be assignable: {:?}",
        result.diagnostics()
    );
}
