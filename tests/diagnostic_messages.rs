use tsrzl::diagnostics;

#[test]
fn should_substitute_argument_given_placeholder_message_when_formatting_diagnostic() {
    // Arrange
    let message = diagnostics::X_0_EXPECTED;

    // Act
    let actual = message.format(&[";"]);

    // Assert
    assert_eq!(actual, "';' expected.");
}
