//! ECMAScript character classes used by the scanner.

use super::identifier_tables::{IDENTIFIER_PART, IDENTIFIER_START};

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

/// Returns whether `character` can start an ECMAScript identifier.
pub(crate) fn is_identifier_start(character: char) -> bool {
    character.is_ascii_alphabetic()
        || matches!(character, '_' | '$')
        || (!character.is_ascii() && in_table(IDENTIFIER_START, character))
}

/// Returns whether `character` can continue an ECMAScript identifier.
pub(crate) fn is_identifier_part(character: char) -> bool {
    character.is_ascii_alphanumeric()
        || matches!(character, '_' | '$')
        || (!character.is_ascii() && in_table(IDENTIFIER_PART, character))
}

fn in_table(table: &[(u32, u32, u32)], character: char) -> bool {
    let value = u32::from(character);
    let index = table.partition_point(|&(low, _, _)| low <= value);
    index > 0 && {
        let (low, high, stride) = table[index - 1];
        value <= high && (value - low) % stride == 0
    }
}

/// Returns whether `character` is any ECMAScript whitespace or line terminator.
pub(crate) const fn is_white_space_like(character: char) -> bool {
    is_white_space_single_line(character) || is_line_break(character)
}
