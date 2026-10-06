use tsrzl::bundled::{lib_names, lib_text};

#[test]
fn should_list_libs_in_name_order_given_bundled_libs_when_reading_lib_names() {
    // Arrange
    let names = lib_names();

    // Act
    let sorted = names.windows(2).all(|pair| pair[0] < pair[1]);

    // Assert
    assert!(sorted && names.len() == 108);
}

#[test]
fn should_return_declarations_given_es5_lib_name_when_reading_lib_text() {
    // Arrange
    let name = "lib.es5.d.ts";

    // Act
    let text = lib_text(name);

    // Assert
    assert!(text.is_some_and(|text| text.contains("interface Array<T>")));
}

#[test]
fn should_return_none_given_unknown_lib_name_when_reading_lib_text() {
    // Arrange
    let name = "lib.unknown.d.ts";

    // Act
    let text = lib_text(name);

    // Assert
    assert_eq!(text, None);
}
