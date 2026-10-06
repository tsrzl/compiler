use tsrzl::check::{ObjectFlags, TypeFlags};

#[test]
fn should_include_null_given_primitive_when_reading_type_flags() {
    // Arrange
    let primitive = TypeFlags::PRIMITIVE;

    // Act
    let includes_null = primitive.contains(TypeFlags::NULL);

    // Assert
    assert!(includes_null);
}

#[test]
fn should_resolve_later_declared_kinds_given_object_type_kind_mask_when_reading_object_flags() {
    // Arrange
    let mask = ObjectFlags::OBJECT_TYPE_KIND_MASK;

    // Act
    let includes_instantiation_expression =
        mask.contains(ObjectFlags::INSTANTIATION_EXPRESSION_TYPE);

    // Assert
    assert!(includes_instantiation_expression);
}
