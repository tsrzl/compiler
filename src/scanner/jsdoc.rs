//! JSDoc comment text and tag token scanning.

use crate::ast::{SyntaxKind, TokenFlags};

use super::Scanner;
use super::chars::{
    is_identifier_part, is_identifier_start, is_line_break, is_white_space_single_line,
};
use super::keywords::identifier_token;

impl Scanner<'_> {
    /// Scans JSDoc comment text up to a line break, backtick, `{`, or tag start; falls back to
    /// [`Self::scan_jsdoc_token`] when no text is present.
    pub fn scan_jsdoc_comment_text_token(&mut self, in_backticks: bool) -> SyntaxKind {
        self.state.begin_token();
        if self.byte_at(0).is_none() {
            self.state.finish_token(SyntaxKind::EndOfFile);
            return SyntaxKind::EndOfFile;
        }
        self.state.begin_token_text();
        while let Some(character) = self.char_at_pos() {
            if is_line_break(character) || character == '`' {
                break;
            }
            if !in_backticks && (character == '{' || self.starts_jsdoc_tag(character)) {
                break;
            }
            self.state.advance(character.len_utf8());
        }
        if self.state.pos() == self.state.token_start() {
            return self.scan_jsdoc_token();
        }
        let value = self.text[self.state.token_start()..self.state.pos()].to_owned();
        self.state.set_token_value(value);
        self.state.finish_token(SyntaxKind::JSDocCommentTextToken);
        SyntaxKind::JSDocCommentTextToken
    }

    /// Returns whether a JSDoc tag name can follow an `@` at the current position.
    #[must_use]
    pub fn can_follow_jsdoc_at(&self) -> bool {
        self.char_at_pos().is_none_or(|character| {
            is_identifier_start(character)
                || is_white_space_single_line(character)
                || is_line_break(character)
        })
    }

    /// Scans one token inside a JSDoc comment.
    pub fn scan_jsdoc_token(&mut self) -> SyntaxKind {
        self.state.begin_token();
        let Some(character) = self.char_at_pos() else {
            self.state.finish_token(SyntaxKind::EndOfFile);
            return SyntaxKind::EndOfFile;
        };
        self.state.begin_token_text();
        self.state.advance(character.len_utf8());
        let token = match character {
            '\t' | '\u{b}' | '\u{c}' | ' ' => {
                while let Some(next) = self
                    .char_at_pos()
                    .filter(|&c| is_white_space_single_line(c))
                {
                    self.state.advance(next.len_utf8());
                }
                SyntaxKind::WhitespaceTrivia
            }
            '\r' | '\n' => {
                if character == '\r' && self.byte_at(0) == Some(b'\n') {
                    self.state.advance(1);
                }
                self.state.add_flags(TokenFlags::PRECEDING_LINE_BREAK);
                SyntaxKind::NewLineTrivia
            }
            '@' => SyntaxKind::AtToken,
            '*' => SyntaxKind::AsteriskToken,
            '{' => SyntaxKind::OpenBraceToken,
            '}' => SyntaxKind::CloseBraceToken,
            '[' => SyntaxKind::OpenBracketToken,
            ']' => SyntaxKind::CloseBracketToken,
            '(' => SyntaxKind::OpenParenToken,
            ')' => SyntaxKind::CloseParenToken,
            '<' => SyntaxKind::LessThanToken,
            '>' => SyntaxKind::GreaterThanToken,
            '=' => SyntaxKind::EqualsToken,
            ',' => SyntaxKind::CommaToken,
            '.' => SyntaxKind::DotToken,
            '`' => SyntaxKind::BacktickToken,
            '#' => SyntaxKind::HashToken,
            '\\' => self.scan_jsdoc_escaped_identifier(),
            _ if is_identifier_start(character) => self.scan_jsdoc_identifier(character),
            _ => SyntaxKind::Unknown,
        };
        self.state.finish_token(token);
        token
    }

    /// Returns whether an `@` at the current position starts a tag: it must follow single-line
    /// whitespace and precede an identifier start.
    fn starts_jsdoc_tag(&self, character: char) -> bool {
        if character != '@' {
            return false;
        }
        let pos = self.state.pos();
        let follows_white_space = self.text[..pos]
            .chars()
            .next_back()
            .is_some_and(is_white_space_single_line);
        follows_white_space
            && self.text[pos + 1..]
                .chars()
                .next()
                .is_some_and(is_identifier_start)
    }

    fn scan_jsdoc_escaped_identifier(&mut self) -> SyntaxKind {
        self.state.retreat(1);
        match self.peek_unicode_escape().and_then(char::from_u32) {
            Some(start) if is_identifier_start(start) => {
                self.scan_unicode_escape(true);
                let value = format!("{start}{}", self.scan_identifier_parts());
                let token = identifier_token(&value);
                self.state.set_token_value(value);
                token
            }
            _ => {
                self.state.advance(1);
                SyntaxKind::Unknown
            }
        }
    }

    fn scan_jsdoc_identifier(&mut self, first: char) -> SyntaxKind {
        let mut last = first;
        while let Some(character) = self.char_at_pos() {
            last = character;
            if !is_identifier_part(character) && character != '-' {
                break;
            }
            self.state.advance(character.len_utf8());
        }
        let mut value = self.text[self.state.token_start()..self.state.pos()].to_owned();
        if last == '\\' {
            value.push_str(&self.scan_identifier_parts());
        }
        let token = identifier_token(&value);
        self.state.set_token_value(value);
        token
    }
}
