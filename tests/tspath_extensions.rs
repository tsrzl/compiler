use tsrzl::tspath::remove_file_extension;

#[test]
fn should_remove_declaration_extension_given_dts_path_when_removing_file_extension() {
    // Arrange
    let path = "src/types.d.ts";

    // Act
    let actual = remove_file_extension(path);

    // Assert
    assert_eq!(actual, "src/types");
}

#[test]
fn should_remove_module_extension_given_mjs_path_when_removing_file_extension() {
    // Arrange
    let path = "src/feature.mjs";

    // Act
    let actual = remove_file_extension(path);

    // Assert
    assert_eq!(actual, "src/feature");
}

#[test]
fn should_keep_path_given_unknown_extension_when_removing_file_extension() {
    // Arrange
    let path = "styles/site.css";

    // Act
    let actual = remove_file_extension(path);

    // Assert
    assert_eq!(actual, "styles/site.css");
}
