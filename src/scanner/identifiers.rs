//! Identifier and private-identifier scanning.

use crate::ast::SyntaxKind;
use crate::diagnostics;

use super::Scanner;
use super::chars::{is_identifier_part, is_identifier_start};
use super::keywords::identifier_token;

impl Scanner<'_> {
    /// Scans an identifier that starts with a unicode escape at a backslash.
    pub(super) fn scan_escaped_identifier(&mut self) -> SyntaxKind {
        match self.peek_unicode_escape().and_then(char::from_u32) {
            Some(character) if is_identifier_start(character) => {
                self.scan_unicode_escape(true);
                let mut value = character.to_string();
                value.push_str(&self.scan_identifier_parts());
                let token = identifier_token(&value);
                self.state.token_value = value;
                token
            }
            _ => {
                self.scan_invalid_character();
                SyntaxKind::Unknown
            }
        }
    }

    /// Scans identifier parts, cooking unicode escapes, and returns the cooked text.
    pub(super) fn scan_identifier_parts(&mut self) -> String {
        let mut value = String::new();
        let mut start = self.state.pos;
        while let Some(character) = self.char_at_pos() {
            if is_identifier_part(character) {
                self.state.pos += character.len_utf8();
                continue;
            }
            if character == '\\'
                && let Some(escaped) = self.peek_unicode_escape().and_then(char::from_u32)
                && is_identifier_part(escaped)
            {
                value.push_str(&self.text[start..self.state.pos]);
                self.scan_unicode_escape(true);
                value.push(escaped);
                start = self.state.pos;
                continue;
            }
            break;
        }
        value.push_str(&self.text[start..self.state.pos]);
        value
    }

    /// Scans an identifier after `prefix_length` bytes and stores its text as the token value.
    pub(super) fn scan_identifier(&mut self, prefix_length: usize) -> bool {
        let start = self.state.pos;
        self.state.pos += prefix_length;
        match self.char_at_pos() {
            Some(character) if is_identifier_start(character) => {
                self.state.pos += character.len_utf8();
            }
            _ => return false,
        }
        let head = &self.text[start..self.state.pos];
        let tail = self.scan_identifier_parts();
        self.state.token_value = format!("{head}{tail}");
        true
    }

    /// Scans a `#` private identifier, or a shebang at the start of the file.
    ///
    /// Returns `None` when a shebang was skipped as trivia.
    pub(super) fn scan_hash(&mut self) -> Option<SyntaxKind> {
        if self.byte_at(1) == Some(b'!') {
            if self.state.pos == 0 {
                self.state.pos = super::trivia::scan_shebang_trivia(self.text, 0);
                return None;
            }
            self.error_at(
                diagnostics::X_CAN_ONLY_BE_USED_AT_THE_START_OF_A_FILE,
                self.state.pos,
                2,
                &[],
            );
            self.state.pos += 1;
            return Some(SyntaxKind::Unknown);
        }
        if self.byte_at(1) == Some(b'\\') {
            self.state.pos += 1;
            if let Some(character) = self.peek_unicode_escape().and_then(char::from_u32)
                && is_identifier_start(character)
            {
                self.scan_unicode_escape(true);
                let parts = self.scan_identifier_parts();
                self.state.token_value = format!("#{character}{parts}");
                return Some(SyntaxKind::PrivateIdentifier);
            }
            self.state.pos -= 1;
        }
        if !self.scan_identifier(1) {
            self.error_at(diagnostics::INVALID_CHARACTER, self.state.pos - 1, 1, &[]);
            self.state.token_value = "#".to_owned();
        }
        Some(SyntaxKind::PrivateIdentifier)
    }
}
