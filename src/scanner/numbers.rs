//! Numeric and bigint literal scanning.

use crate::ast::{SyntaxKind, TokenFlags};
use crate::diagnostics;
use crate::jsnum::{from_string, number_to_string, parse_pseudo_big_int};

use super::Scanner;
use super::chars::is_identifier_start;

impl Scanner<'_> {
    /// Scans a `0x`, `0b`, or `0o` prefixed literal at the current position, if present.
    pub(super) fn scan_prefixed_number(&mut self) -> Option<SyntaxKind> {
        let (radix, flag, missing_digits) = match self.byte_at(1)? {
            b'x' | b'X' => (
                16,
                TokenFlags::HEX_SPECIFIER,
                diagnostics::HEXADECIMAL_DIGIT_EXPECTED,
            ),
            b'b' | b'B' => (
                2,
                TokenFlags::BINARY_SPECIFIER,
                diagnostics::BINARY_DIGIT_EXPECTED,
            ),
            b'o' | b'O' => (
                8,
                TokenFlags::OCTAL_SPECIFIER,
                diagnostics::OCTAL_DIGIT_EXPECTED,
            ),
            _ => return None,
        };
        self.state.pos += 2;
        let mut digits = if radix == 16 {
            self.scan_hex_digits(1, true, true)
        } else {
            self.scan_binary_or_octal_digits(radix)
        };
        if digits.is_empty() {
            self.error(missing_digits);
            digits = "0".to_owned();
        }
        let prefix = match radix {
            16 => "0x",
            2 => "0b",
            _ => "0o",
        };
        self.state.token_value = format!("{prefix}{digits}");
        self.state.token_flags |= flag;
        Some(self.scan_big_int_suffix())
    }

    /// Scans a decimal numeric literal, including a leading-dot fraction.
    pub(super) fn scan_number(&mut self) -> SyntaxKind {
        let mut start = self.state.pos;
        let fixed_part = if self.byte_at(0) == Some(b'0') {
            self.state.pos += 1;
            if self.byte_at(0) == Some(b'_') {
                self.state.token_flags |=
                    TokenFlags::CONTAINS_SEPARATOR | TokenFlags::CONTAINS_INVALID_SEPARATOR;
                self.error_at(
                    diagnostics::NUMERIC_SEPARATORS_ARE_NOT_ALLOWED_HERE,
                    self.state.pos,
                    1,
                    &[],
                );
                self.state.pos = start;
                self.scan_number_fragment()
            } else {
                let (digits, is_octal) = self.scan_digits();
                if digits.is_empty() {
                    "0".to_owned()
                } else if !is_octal {
                    self.state.token_flags |= TokenFlags::CONTAINS_LEADING_ZERO;
                    digits
                } else {
                    let value = u64::from_str_radix(&digits, 8).unwrap_or(u64::MAX);
                    #[allow(clippy::cast_precision_loss)]
                    let number = value as f64;
                    self.state.token_value = number_to_string(number);
                    self.state.token_flags |= TokenFlags::OCTAL;
                    let with_minus = self.state.token == SyntaxKind::MinusToken;
                    let literal = format!("{}0o{value:o}", if with_minus { "-" } else { "" });
                    if with_minus {
                        start -= 1;
                    }
                    self.error_at(
                        diagnostics::OCTAL_LITERALS_ARE_NOT_ALLOWED_USE_THE_SYNTAX_0,
                        start,
                        self.state.pos - start,
                        &[&literal],
                    );
                    return SyntaxKind::NumericLiteral;
                }
            }
        } else {
            self.scan_number_fragment()
        };
        let fixed_part_end = self.state.pos;
        let mut fractional_part = String::new();
        let mut exponent_preamble = "";
        let mut exponent_part = String::new();
        if self.byte_at(0) == Some(b'.') {
            self.state.pos += 1;
            fractional_part = self.scan_number_fragment();
        }
        let mut end = self.state.pos;
        if matches!(self.byte_at(0), Some(b'e' | b'E')) {
            self.state.pos += 1;
            self.state.token_flags |= TokenFlags::SCIENTIFIC;
            if matches!(self.byte_at(0), Some(b'+' | b'-')) {
                self.state.pos += 1;
            }
            let numeric_start = self.state.pos;
            exponent_part = self.scan_number_fragment();
            if exponent_part.is_empty() {
                self.error(diagnostics::DIGIT_EXPECTED);
            } else {
                exponent_preamble = &self.text[end..numeric_start];
                end = self.state.pos;
            }
        }
        self.state.token_value = if self
            .state
            .token_flags
            .intersects(TokenFlags::CONTAINS_SEPARATOR)
        {
            let mut value = fixed_part;
            if !fractional_part.is_empty() {
                value.push('.');
                value.push_str(&fractional_part);
            }
            if !exponent_part.is_empty() {
                value.push_str(exponent_preamble);
                value.push_str(&exponent_part);
            }
            value
        } else {
            self.text[start..end].to_owned()
        };
        if self
            .state
            .token_flags
            .intersects(TokenFlags::CONTAINS_LEADING_ZERO)
        {
            self.error_at(
                diagnostics::DECIMALS_WITH_LEADING_ZEROS_ARE_NOT_ALLOWED,
                start,
                self.state.pos - start,
                &[],
            );
            self.state.token_value = number_to_string(from_string(&self.state.token_value));
            return SyntaxKind::NumericLiteral;
        }
        let result = if fixed_part_end == self.state.pos {
            self.scan_big_int_suffix()
        } else {
            self.state.token_value = number_to_string(from_string(&self.state.token_value));
            SyntaxKind::NumericLiteral
        };
        self.check_identifier_after_number(start, fixed_part_end, result)
    }

    fn check_identifier_after_number(
        &mut self,
        start: usize,
        fixed_part_end: usize,
        result: SyntaxKind,
    ) -> SyntaxKind {
        if !self.char_at_pos().is_some_and(is_identifier_start) {
            return result;
        }
        let identifier_start = self.state.pos;
        let identifier = self.scan_identifier_parts();
        if result != SyntaxKind::BigIntLiteral && identifier == "n" {
            if self.state.token_flags.intersects(TokenFlags::SCIENTIFIC) {
                self.error_at(
                    diagnostics::A_BIGINT_LITERAL_CANNOT_USE_EXPONENTIAL_NOTATION,
                    start,
                    self.state.pos - start,
                    &[],
                );
                return result;
            }
            if fixed_part_end < identifier_start {
                self.error_at(
                    diagnostics::A_BIGINT_LITERAL_MUST_BE_AN_INTEGER,
                    start,
                    self.state.pos - start,
                    &[],
                );
                return result;
            }
        }
        self.error_at(
            diagnostics::AN_IDENTIFIER_OR_KEYWORD_CANNOT_IMMEDIATELY_FOLLOW_A_NUMERIC_LITERAL,
            identifier_start,
            self.state.pos - identifier_start,
            &[],
        );
        self.state.pos = identifier_start;
        result
    }

    fn scan_number_fragment(&mut self) -> String {
        let mut start = self.state.pos;
        let mut allow_separator = false;
        let mut previous_was_separator = false;
        let mut result = String::new();
        loop {
            let before = self.state.pos;
            while self.byte_at(0).is_some_and(|byte| byte.is_ascii_digit()) {
                self.state.pos += 1;
            }
            if self.state.pos > before {
                allow_separator = true;
                previous_was_separator = false;
            }
            if self.byte_at(0) != Some(b'_') {
                break;
            }
            self.state.token_flags |= TokenFlags::CONTAINS_SEPARATOR;
            if allow_separator {
                allow_separator = false;
                previous_was_separator = true;
                result.push_str(&self.text[start..self.state.pos]);
            } else {
                self.state.token_flags |= TokenFlags::CONTAINS_INVALID_SEPARATOR;
                self.report_misplaced_separator(previous_was_separator, self.state.pos);
            }
            self.state.pos += 1;
            start = self.state.pos;
        }
        if previous_was_separator {
            self.state.token_flags |= TokenFlags::CONTAINS_INVALID_SEPARATOR;
            self.error_at(
                diagnostics::NUMERIC_SEPARATORS_ARE_NOT_ALLOWED_HERE,
                self.state.pos - 1,
                1,
                &[],
            );
        }
        result.push_str(&self.text[start..self.state.pos]);
        result
    }

    fn scan_digits(&mut self) -> (String, bool) {
        let start = self.state.pos;
        let mut is_octal = true;
        while let Some(byte) = self.byte_at(0).filter(u8::is_ascii_digit) {
            is_octal &= byte <= b'7';
            self.state.pos += 1;
        }
        (self.text[start..self.state.pos].to_owned(), is_octal)
    }

    fn scan_binary_or_octal_digits(&mut self, radix: u32) -> String {
        let mut digits = String::new();
        let mut allow_separator = false;
        let mut previous_was_separator = false;
        loop {
            match self.byte_at(0) {
                Some(byte) if char::from(byte).is_digit(radix) => {
                    digits.push(char::from(byte));
                    allow_separator = true;
                    previous_was_separator = false;
                }
                Some(b'_') => {
                    self.state.token_flags |= TokenFlags::CONTAINS_SEPARATOR;
                    if allow_separator {
                        allow_separator = false;
                        previous_was_separator = true;
                    } else {
                        self.report_misplaced_separator(previous_was_separator, self.state.pos);
                    }
                }
                _ => break,
            }
            self.state.pos += 1;
        }
        if previous_was_separator {
            self.error_at(
                diagnostics::NUMERIC_SEPARATORS_ARE_NOT_ALLOWED_HERE,
                self.state.pos - 1,
                1,
                &[],
            );
        }
        digits
    }

    fn scan_big_int_suffix(&mut self) -> SyntaxKind {
        if self.byte_at(0) == Some(b'n') {
            self.state.token_value.push('n');
            if self
                .state
                .token_flags
                .intersects(TokenFlags::BINARY_OR_OCTAL_SPECIFIER)
            {
                self.state.token_value =
                    format!("{}n", parse_pseudo_big_int(&self.state.token_value));
            }
            self.state.pos += 1;
            return SyntaxKind::BigIntLiteral;
        }
        self.state.token_value = number_to_string(from_string(&self.state.token_value));
        SyntaxKind::NumericLiteral
    }
}
