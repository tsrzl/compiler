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

#[test]
fn should_return_utf8_byte_offset_given_utf16_offset_after_astral_character_when_converting_offsets()
 {
    // Arrange
    let source = SourceText::new("a😀b");
    let offset = Utf16Offset::new(3);

    // Act
    let actual = source.utf8_offset(offset);

    // Assert
    assert_eq!(actual, Some(5));
}
