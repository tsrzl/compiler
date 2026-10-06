//! String literal, escape sequence, and hexadecimal digit scanning.

use crate::ast::TokenFlags;
use crate::diagnostics;

use super::Scanner;

/// Options controlling how an escape sequence is cooked and reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct EscapeOptions {
    pub(super) report_errors: bool,
    pub(super) allow_extended_unicode: bool,
}

impl EscapeOptions {
    pub(super) const STRING: Self = Self {
        report_errors: true,
        allow_extended_unicode: true,
    };
}

const LEADING_SURROGATES: std::ops::RangeInclusive<u32> = 0xD800..=0xDBFF;
const TRAILING_SURROGATES: std::ops::RangeInclusive<u32> = 0xDC00..=0xDFFF;

impl Scanner<'_> {
    /// Scans a quoted string at the current position and returns its cooked value.
    pub(super) fn scan_string(&mut self, jsx_attribute_string: bool) -> String {
        let quote = self.byte_at(0).unwrap_or(b'"');
        if quote == b'\'' {
            self.state.add_flags(TokenFlags::SINGLE_QUOTE);
        }
        self.state.advance(1);
        let mut value = String::new();
        let mut start = self.state.pos();
        loop {
            let Some(byte) = self.byte_at(0) else {
                value.push_str(&self.text[start..self.state.pos()]);
                self.state.add_flags(TokenFlags::UNTERMINATED);
                self.error(diagnostics::UNTERMINATED_STRING_LITERAL);
                break;
            };
            if byte == quote {
                value.push_str(&self.text[start..self.state.pos()]);
                self.state.advance(1);
                break;
            }
            if byte == b'\\' && !jsx_attribute_string {
                value.push_str(&self.text[start..self.state.pos()]);
                value.push_str(&self.scan_escape_sequence(EscapeOptions::STRING));
                start = self.state.pos();
                continue;
            }
            if matches!(byte, b'\n' | b'\r') && !jsx_attribute_string {
                value.push_str(&self.text[start..self.state.pos()]);
                self.state.add_flags(TokenFlags::UNTERMINATED);
                self.error(diagnostics::UNTERMINATED_STRING_LITERAL);
                break;
            }
            self.state.advance(1);
        }
        value
    }

    /// Scans an escape sequence starting at a backslash and returns its cooked text.
    ///
    /// Lone surrogate escapes cook to U+FFFD because Rust strings cannot hold them.
    pub(super) fn scan_escape_sequence(&mut self, options: EscapeOptions) -> String {
        let start = self.state.pos();
        self.state.advance(1);
        let Some(character) = self.char_at_pos() else {
            self.error(diagnostics::UNEXPECTED_END_OF_TEXT);
            return String::new();
        };
        self.state.advance(character.len_utf8());
        match character {
            '0' if !self.byte_at(0).is_some_and(|byte| byte.is_ascii_digit()) => "\0".to_owned(),
            '0'..='7' => self.scan_legacy_octal_escape(start, character, options),
            '8' | '9' => {
                self.state.add_flags(TokenFlags::CONTAINS_INVALID_ESCAPE);
                if options.report_errors {
                    let text = &self.text[start..self.state.pos()];
                    self.error_at(
                        diagnostics::ESCAPE_SEQUENCE_0_IS_NOT_ALLOWED,
                        start,
                        self.state.pos() - start,
                        &[text],
                    );
                    return character.to_string();
                }
                self.text[start..self.state.pos()].to_owned()
            }
            'b' => "\u{8}".to_owned(),
            't' => "\t".to_owned(),
            'n' => "\n".to_owned(),
            'v' => "\u{b}".to_owned(),
            'f' => "\u{c}".to_owned(),
            'r' => "\r".to_owned(),
            '\'' => "'".to_owned(),
            '"' => "\"".to_owned(),
            'u' => self.scan_unicode_escape_value(start, options),
            'x' => self.scan_hex_escape(start, options),
            '\r' => {
                if self.byte_at(0) == Some(b'\n') {
                    self.state.advance(1);
                }
                String::new()
            }
            '\n' | '\u{2028}' | '\u{2029}' => String::new(),
            _ => character.to_string(),
        }
    }

    fn scan_legacy_octal_escape(
        &mut self,
        start: usize,
        first: char,
        options: EscapeOptions,
    ) -> String {
        let is_octal_digit =
            |byte: Option<u8>| byte.is_some_and(|byte| (b'0'..=b'7').contains(&byte));
        if first <= '3' && is_octal_digit(self.byte_at(0)) {
            self.state.advance(1);
        }
        if is_octal_digit(self.byte_at(0)) {
            self.state.advance(1);
        }
        self.state.add_flags(TokenFlags::CONTAINS_INVALID_ESCAPE);
        if options.report_errors {
            let code = u32::from_str_radix(&self.text[start + 1..self.state.pos()], 8)
                .expect("octal escape digits were validated");
            let replacement = format!("\\x{code:02x}");
            self.error_at(
                diagnostics::OCTAL_ESCAPE_SEQUENCES_ARE_NOT_ALLOWED_USE_THE_SYNTAX_0,
                start,
                self.state.pos() - start,
                &[&replacement],
            );
            return char::from_u32(code).map(String::from).unwrap_or_default();
        }
        self.text[start..self.state.pos()].to_owned()
    }

    fn scan_hex_escape(&mut self, start: usize, options: EscapeOptions) -> String {
        while self.state.pos() < start + 4 {
            if !self.byte_at(0).is_some_and(|byte| byte.is_ascii_hexdigit()) {
                self.state.add_flags(TokenFlags::CONTAINS_INVALID_ESCAPE);
                if options.report_errors {
                    self.error(diagnostics::HEXADECIMAL_DIGIT_EXPECTED);
                }
                return self.text[start..self.state.pos()].to_owned();
            }
            self.state.advance(1);
        }
        self.state.add_flags(TokenFlags::HEX_ESCAPE);
        let value = u32::from_str_radix(&self.text[start + 2..self.state.pos()], 16)
            .expect("hex escape digits were validated");
        char::from_u32(value).map(String::from).unwrap_or_default()
    }

    fn scan_unicode_escape_value(&mut self, start: usize, options: EscapeOptions) -> String {
        let extended = self.byte_at(0) == Some(b'{');
        self.state.retreat(2);
        let code_point = self.scan_unicode_escape(options.report_errors);
        if extended && !options.allow_extended_unicode {
            self.state.add_flags(TokenFlags::CONTAINS_INVALID_ESCAPE);
        }
        let Some(code_point) = code_point else {
            return self.text[start..self.state.pos()].to_owned();
        };
        if LEADING_SURROGATES.contains(&code_point)
            && let Some(combined) = self.scan_trailing_surrogate_escape(code_point)
        {
            return combined.to_string();
        }
        char::from_u32(code_point)
            .unwrap_or(char::REPLACEMENT_CHARACTER)
            .to_string()
    }

    /// Scans `\uXXXX` or `\u{X...}` at a backslash and returns its code point when valid.
    pub(super) fn scan_unicode_escape(&mut self, report_errors: bool) -> Option<u32> {
        self.state.advance(2);
        let start = self.state.pos();
        let extended = self.byte_at(0) == Some(b'{');
        let digits = if extended {
            self.state.advance(1);
            self.scan_hex_digits(1, true, false)
        } else {
            self.state.add_flags(TokenFlags::UNICODE_ESCAPE);
            self.scan_hex_digits(4, false, false)
        };
        if digits.is_empty() {
            self.state.add_flags(TokenFlags::CONTAINS_INVALID_ESCAPE);
            if report_errors {
                self.error(diagnostics::HEXADECIMAL_DIGIT_EXPECTED);
            }
            return None;
        }
        let value = u32::from_str_radix(&digits, 16).unwrap_or(u32::MAX);
        if !extended {
            return Some(value);
        }
        let mut invalid = false;
        if value > 0x0010_FFFF {
            if report_errors {
                self.error_at(
                    diagnostics::AN_EXTENDED_UNICODE_ESCAPE_VALUE_MUST_BE_BETWEEN_0X0_AND_0X10FFFF_INCLUSIVE,
                    start + 1,
                    self.state.pos() - start - 1,
                    &[],
                );
            }
            invalid = true;
        }
        match self.byte_at(0) {
            None => {
                if report_errors {
                    self.error(diagnostics::UNEXPECTED_END_OF_TEXT);
                }
                invalid = true;
            }
            Some(b'}') => self.state.advance(1),
            Some(_) => {
                if report_errors {
                    self.error(diagnostics::UNTERMINATED_UNICODE_ESCAPE_SEQUENCE);
                }
                invalid = true;
            }
        }
        if invalid {
            self.state.add_flags(TokenFlags::CONTAINS_INVALID_ESCAPE);
            return None;
        }
        self.state.add_flags(TokenFlags::EXTENDED_UNICODE_ESCAPE);
        Some(value)
    }

    fn scan_trailing_surrogate_escape(&mut self, leading: u32) -> Option<char> {
        if self.byte_at(0) != Some(b'\\') || self.byte_at(1) != Some(b'u') {
            return None;
        }
        let saved_pos = self.state.pos();
        let saved_flags = self.state.flags();
        if let Some(trailing) = self.scan_unicode_escape(false)
            && TRAILING_SURROGATES.contains(&trailing)
        {
            return char::from_u32(0x10000 + ((leading - 0xD800) << 10) + (trailing - 0xDC00));
        }
        self.state.set_pos(saved_pos);
        self.state.set_flags(saved_flags);
        None
    }

    /// Returns the code point of a valid unicode escape at a backslash without consuming it.
    pub(super) fn peek_unicode_escape(&mut self) -> Option<u32> {
        if self.byte_at(1) != Some(b'u') {
            return None;
        }
        let saved_pos = self.state.pos();
        let saved_flags = self.state.flags();
        let code_point = self.scan_unicode_escape(false);
        self.state.set_pos(saved_pos);
        self.state.set_flags(saved_flags);
        code_point
    }

    /// Scans hexadecimal digits and returns them lowercased without separators.
    ///
    /// Returns an empty string when fewer than `min_count` digits are present.
    pub(super) fn scan_hex_digits(
        &mut self,
        min_count: usize,
        scan_as_many_as_possible: bool,
        can_have_separators: bool,
    ) -> String {
        let start = self.state.pos();
        let mut digit_count = 0;
        let mut allow_separator = false;
        let mut previous_was_separator = false;
        while digit_count < min_count || scan_as_many_as_possible {
            match self.byte_at(0) {
                Some(byte) if byte.is_ascii_hexdigit() => {
                    allow_separator = can_have_separators;
                    previous_was_separator = false;
                    digit_count += 1;
                }
                Some(b'_') if can_have_separators => {
                    self.state.add_flags(TokenFlags::CONTAINS_SEPARATOR);
                    if allow_separator {
                        allow_separator = false;
                        previous_was_separator = true;
                    } else {
                        self.report_misplaced_separator(previous_was_separator, self.state.pos());
                    }
                }
                _ => break,
            }
            self.state.advance(1);
        }
        if previous_was_separator {
            self.error_at(
                diagnostics::NUMERIC_SEPARATORS_ARE_NOT_ALLOWED_HERE,
                self.state.pos() - 1,
                1,
                &[],
            );
        }
        if digit_count < min_count {
            return String::new();
        }
        self.text[start..self.state.pos()]
            .chars()
            .filter(|&character| character != '_')
            .map(|character| character.to_ascii_lowercase())
            .collect()
    }

    pub(super) fn report_misplaced_separator(&mut self, consecutive: bool, pos: usize) {
        let message = if consecutive {
            diagnostics::MULTIPLE_CONSECUTIVE_NUMERIC_SEPARATORS_ARE_NOT_PERMITTED
        } else {
            diagnostics::NUMERIC_SEPARATORS_ARE_NOT_ALLOWED_HERE
        };
        self.error_at(message, pos, 1, &[]);
    }
}
