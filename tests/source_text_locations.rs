use tsrzl::source_text::{LinePosition, SourceText, Utf16Offset};

#[test]
fn should_report_utf16_column_after_astral_character_given_source_position_when_mapping_line_position()
 {
    // Arrange
    let source = SourceText::new("a😀b");
    let offset = Utf16Offset::new(3);

    // Act
    let actual = source.line_position(offset);

    // Assert
    assert_eq!(actual, Some(LinePosition::new(0, 3)));
}
