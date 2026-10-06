//! JSX text, identifier, and attribute-value scanning.

use crate::ast::SyntaxKind;
use crate::diagnostics;

use super::Scanner;
use super::chars::{is_line_break, is_white_space_like};
use super::keywords::identifier_token;
use super::trivia::is_conflict_marker_trivia;

impl Scanner<'_> {
    /// Scans JSX child content, joining multi-line text into one token.
    pub fn scan_jsx_token(&mut self) -> SyntaxKind {
        self.scan_jsx_token_with(true)
    }

    /// Scans JSX child content; with `allow_multiline_text` false, text stops at each line end.
    pub fn scan_jsx_token_with(&mut self, allow_multiline_text: bool) -> SyntaxKind {
        self.state.begin_full_start();
        self.state.begin_token_text();
        let token = match (self.byte_at(0), self.byte_at(1)) {
            (None, _) => SyntaxKind::EndOfFile,
            (Some(b'<'), Some(b'/')) => {
                self.state.advance(2);
                SyntaxKind::LessThanSlashToken
            }
            (Some(b'<'), _) => {
                self.state.advance(1);
                SyntaxKind::LessThanToken
            }
            (Some(b'{'), _) => {
                self.state.advance(1);
                SyntaxKind::OpenBraceToken
            }
            _ => self.scan_jsx_text(allow_multiline_text),
        };
        self.state.finish_token(token);
        token
    }

    /// Rescans from the current token's full start as JSX child content.
    pub fn rescan_jsx_token(&mut self, allow_multiline_text: bool) -> SyntaxKind {
        self.state.set_pos(self.state.full_start());
        self.state.begin_token_text();
        self.scan_jsx_token_with(allow_multiline_text)
    }

    fn scan_jsx_text(&mut self, allow_multiline_text: bool) -> SyntaxKind {
        // Mirrors TypeScript-Go: 0 until non-whitespace is seen, -1 after a line break in leading
        // whitespace, otherwise the position of the last non-whitespace character.
        let mut first_non_white_space: isize = 0;
        while let Some(character) = self.char_at_pos() {
            if character == '{' {
                break;
            }
            if character == '<' {
                if is_conflict_marker_trivia(self.text, self.state.pos()) {
                    self.skip_conflict_marker();
                    return SyntaxKind::ConflictMarkerTrivia;
                }
                break;
            }
            let pos = self.state.pos();
            if character == '>' {
                self.error_at(
                    diagnostics::UNEXPECTED_TOKEN_DID_YOU_MEAN_OR_GT,
                    pos,
                    1,
                    &[],
                );
            } else if character == '}' {
                self.error_at(
                    diagnostics::UNEXPECTED_TOKEN_DID_YOU_MEAN_OR_RBRACE,
                    pos,
                    1,
                    &[],
                );
            }
            if is_line_break(character) && first_non_white_space == 0 {
                first_non_white_space = -1;
            } else if !allow_multiline_text && is_line_break(character) && first_non_white_space > 0
            {
                break;
            } else if !is_white_space_like(character) {
                first_non_white_space = isize::try_from(pos).expect("source offsets fit in isize");
            }
            self.state.advance(character.len_utf8());
        }
        let value = self.text[self.state.full_start()..self.state.pos()].to_owned();
        self.state.set_token_value(value);
        if first_non_white_space == -1 {
            SyntaxKind::JsxTextAllWhiteSpaces
        } else {
            SyntaxKind::JsxText
        }
    }

    /// Extends an identifier or keyword token with JSX `-` separated parts.
    pub fn scan_jsx_identifier(&mut self) -> SyntaxKind {
        if !self.state.token().is_identifier_or_keyword() {
            return self.state.token();
        }
        let mut value = self.state.token_value().to_owned();
        while let Some(byte) = self.byte_at(0) {
            if byte == b'-' {
                value.push('-');
                self.state.advance(1);
                continue;
            }
            let before = self.state.pos();
            value.push_str(&self.scan_identifier_parts());
            if self.state.pos() == before {
                break;
            }
        }
        let token = identifier_token(&value);
        self.state.set_token_value(value);
        self.state.finish_token(token);
        token
    }

    /// Scans a JSX attribute value, where quoted strings keep backslashes literally.
    pub fn scan_jsx_attribute_value(&mut self) -> SyntaxKind {
        self.state.begin_full_start();
        while let Some(character) = self.char_at_pos().filter(|&c| is_white_space_like(c)) {
            self.state.advance(character.len_utf8());
        }
        self.state.begin_token_text();
        if matches!(self.byte_at(0), Some(b'"' | b'\'')) {
            let value = self.scan_string(true);
            self.state.set_token_value(value);
            self.state.finish_token(SyntaxKind::StringLiteral);
            return SyntaxKind::StringLiteral;
        }
        self.scan()
    }

    /// Rescans from the current token's full start as a JSX attribute value.
    pub fn rescan_jsx_attribute_value(&mut self) -> SyntaxKind {
        self.state.set_pos(self.state.full_start());
        self.state.begin_token_text();
        self.scan_jsx_attribute_value()
    }
}
