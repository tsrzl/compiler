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

#[test]
fn should_map_option_to_file_given_lib_option_name_when_reading_lib_file() {
    // Arrange
    let option = "es2015.promise";

    // Act
    let file = tsrzl::bundled::lib_file_for_option(option);

    // Assert
    assert_eq!(file, Some("lib.es2015.promise.d.ts"));
}

#[test]
fn should_use_es6_lib_given_es2015_target_when_reading_default_lib() {
    // Arrange
    let target = "es2015";

    // Act
    let file = tsrzl::bundled::default_lib_file_name(target);

    // Assert
    assert_eq!(file, "lib.es6.d.ts");
}

#[test]
fn should_use_plain_lib_given_es5_target_when_reading_default_lib() {
    // Arrange
    let target = "es5";

    // Act
    let file = tsrzl::bundled::default_lib_file_name(target);

    // Assert
    assert_eq!(file, "lib.d.ts");
}
