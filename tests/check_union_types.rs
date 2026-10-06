use tsrzl::check::{TypeFlags, TypeTable};

fn table() -> TypeTable {
    TypeTable::new(true)
}

#[test]
fn should_return_never_given_no_constituents_when_creating_union_type() {
    // Arrange
    let mut types = table();

    // Act
    let union = types.union_type(&[]);

    // Assert
    assert_eq!(union, types.intrinsics().never);
}

#[test]
fn should_return_constituent_given_single_type_when_creating_union_type() {
    // Arrange
    let mut types = table();
    let string = types.intrinsics().string;

    // Act
    let union = types.union_type(&[string]);

    // Assert
    assert_eq!(union, string);
}

#[test]
fn should_order_by_type_flags_given_number_before_string_when_creating_union_type() {
    // Arrange
    let mut types = table();
    let (string, number) = (types.intrinsics().string, types.intrinsics().number);

    // Act
    let union = types.union_type(&[number, string]);

    // Assert
    assert_eq!(types.union_constituents(union), [string, number]);
}

#[test]
fn should_reuse_union_given_same_constituents_in_other_order_when_creating_union_type() {
    // Arrange
    let mut types = table();
    let (string, number) = (types.intrinsics().string, types.intrinsics().number);
    let first = types.union_type(&[string, number]);

    // Act
    let second = types.union_type(&[number, string, number]);

    // Assert
    assert_eq!(first, second);
}

#[test]
fn should_return_any_given_any_constituent_when_creating_union_type() {
    // Arrange
    let mut types = table();
    let (any, string) = (types.intrinsics().any, types.intrinsics().string);

    // Act
    let union = types.union_type(&[string, any]);

    // Assert
    assert_eq!(union, any);
}

#[test]
fn should_return_unknown_given_unknown_constituent_when_creating_union_type() {
    // Arrange
    let mut types = table();
    let (unknown, string) = (types.intrinsics().unknown, types.intrinsics().string);

    // Act
    let union = types.union_type(&[string, unknown]);

    // Assert
    assert_eq!(union, unknown);
}

#[test]
fn should_ignore_never_given_never_constituent_when_creating_union_type() {
    // Arrange
    let mut types = table();
    let (never, string) = (types.intrinsics().never, types.intrinsics().string);

    // Act
    let union = types.union_type(&[never, string]);

    // Assert
    assert_eq!(union, string);
}

#[test]
fn should_absorb_string_literal_given_string_constituent_when_creating_union_type() {
    // Arrange
    let mut types = table();
    let string = types.intrinsics().string;
    let literal = types.string_literal_type("a");

    // Act
    let union = types.union_type(&[literal, string]);

    // Assert
    assert_eq!(union, string);
}

#[test]
fn should_order_string_literals_by_value_given_unsorted_literals_when_creating_union_type() {
    // Arrange
    let mut types = table();
    let b = types.string_literal_type("b");
    let a = types.string_literal_type("a");

    // Act
    let union = types.union_type(&[b, a]);

    // Assert
    assert_eq!(types.union_constituents(union), [a, b]);
}

#[test]
fn should_order_number_literals_by_value_given_unsorted_literals_when_creating_union_type() {
    // Arrange
    let mut types = table();
    let ten = types.number_literal_type(10.0);
    let two = types.number_literal_type(2.0);

    // Act
    let union = types.union_type(&[ten, two]);

    // Assert
    assert_eq!(types.union_constituents(union), [two, ten]);
}

#[test]
fn should_drop_fresh_literal_given_its_regular_type_when_creating_union_type() {
    // Arrange
    let mut types = table();
    let regular = types.string_literal_type("a");
    let fresh = types.fresh_literal_type(regular);
    let other = types.string_literal_type("b");

    // Act
    let union = types.union_type(&[fresh, regular, other]);

    // Assert
    assert_eq!(types.union_constituents(union), [regular, other]);
}

#[test]
fn should_flatten_nested_union_given_union_constituent_when_creating_union_type() {
    // Arrange
    let mut types = table();
    let (string, number, bigint) = (
        types.intrinsics().string,
        types.intrinsics().number,
        types.intrinsics().bigint,
    );
    let inner = types.union_type(&[string, number]);

    // Act
    let union = types.union_type(&[inner, bigint]);

    // Assert
    assert_eq!(types.union_constituents(union), [string, number, bigint]);
}

#[test]
fn should_flag_boolean_given_true_and_false_when_creating_union_type() {
    // Arrange
    let mut types = table();
    let (regular_false, regular_true) = (
        types.intrinsics().regular_false,
        types.intrinsics().regular_true,
    );

    // Act
    let union = types.union_type(&[regular_true, regular_false]);

    // Assert
    assert_eq!(union, types.intrinsics().boolean);
}

#[test]
fn should_carry_boolean_flag_given_intrinsic_boolean_when_reading_type() {
    // Arrange
    let types = table();

    // Act
    let flags = types.get(types.intrinsics().boolean).flags();

    // Assert
    assert_eq!(flags, TypeFlags::UNION | TypeFlags::BOOLEAN);
}

#[test]
fn should_reuse_literal_given_same_string_value_when_creating_literal_type() {
    // Arrange
    let mut types = table();
    let first = types.string_literal_type("value");

    // Act
    let second = types.string_literal_type("value");

    // Assert
    assert_eq!(first, second);
}

#[test]
fn should_reuse_nan_literal_given_two_nan_values_when_creating_literal_type() {
    // Arrange
    let mut types = table();
    let first = types.number_literal_type(f64::NAN);

    // Act
    let second = types.number_literal_type(f64::NAN);

    // Assert
    assert_eq!(first, second);
}

#[test]
fn should_widen_lone_null_given_non_strict_null_checks_when_creating_union_type() {
    // Arrange
    let mut types = TypeTable::new(false);
    let null = types.intrinsics().null;

    // Act
    let union = types.union_type(&[null, null]);

    // Assert
    assert_eq!(union, types.intrinsics().null);
}
