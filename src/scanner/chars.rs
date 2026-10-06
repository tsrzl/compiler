//! ECMAScript character classes used by the scanner.

/// Returns whether `character` is whitespace that does not terminate a line.
pub(crate) const fn is_white_space_single_line(character: char) -> bool {
    matches!(
        character,
        ' ' | '\t' | '\u{0b}' | '\u{0c}' | '\u{85}' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200b}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}'
    )
}

/// Returns whether `character` is an ECMAScript line terminator.
pub(crate) const fn is_line_break(character: char) -> bool {
    matches!(character, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}
