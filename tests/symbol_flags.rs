use tsrzl::ast::SymbolFlags;

#[test]
fn should_match_typescript_go_bits_given_class_excludes_when_reading_symbol_flags() {
    // Arrange
    let expected = (SymbolFlags::VALUE | SymbolFlags::TYPE)
        .without(SymbolFlags::VALUE_MODULE | SymbolFlags::INTERFACE | SymbolFlags::FUNCTION);

    // Act
    let actual = SymbolFlags::CLASS_EXCLUDES;

    // Assert
    assert_eq!(actual, expected);
}

#[test]
fn should_exclude_global_lookup_given_all_when_reading_symbol_flags() {
    // Arrange
    let all = SymbolFlags::ALL;

    // Act
    let actual = all.intersects(SymbolFlags::GLOBAL_LOOKUP);

    // Assert
    assert!(!actual);
}

#[test]
fn should_complement_supported_default_modifiers_given_unsupported_set_when_reading_symbol_flags() {
    // Arrange
    let supported = SymbolFlags::EXPORT_SUPPORTS_DEFAULT_MODIFIER;

    // Act
    let actual = SymbolFlags::EXPORT_DOES_NOT_SUPPORT_DEFAULT_MODIFIER.bits();

    // Assert
    assert_eq!(actual, !supported.bits());
}
