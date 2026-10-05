use std::path::Path;

use tsrzl::compiler::Compiler;
use tsrzl::source_file::SourceFile;

#[test]
fn should_assign_derived_instance_to_base_return_type_given_class_inheritance_when_checking_types()
{
    // TypeScript 7.0.2 case: conformance/types/typeRelationships/assignmentCompatibility/unionTypesAssignability.ts
    // Oracle expectation: a derived instance is assignable to its base class type.
    // Arrange
    let source = SourceFile::from_path(
        Path::new("unionTypesAssignability.ts"),
        "class Base {} class Derived extends Base {} function acceptBase(value: Derived): Base { return value; }",
    )
    .expect("the TypeScript path has a supported source kind");

    // Act
    let result = Compiler::new().compile(source);

    // Assert
    assert_eq!(result.diagnostics(), []);
}
