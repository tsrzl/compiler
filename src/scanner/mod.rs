//! The TypeScript scanner, ported from TypeScript-Go's `internal/scanner` package.
//!
//! Positions are UTF-8 byte offsets into the source text, matching TypeScript-Go. Convert them to
//! UTF-16 offsets with [`crate::source_text::SourceText`] at reporting boundaries.

mod chars;
mod identifier_tables;
mod keywords;
mod strings;

use crate::ast::{SyntaxKind, TokenFlags};
use crate::diagnostics::{self, Message};

use chars::{is_identifier_part, is_identifier_start, is_line_break, is_white_space_single_line};
pub use keywords::{identifier_token, keyword};

/// A diagnostic reported while scanning, located by UTF-8 byte offsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanDiagnostic {
    message: Message,
    start: usize,
    length: usize,
    arguments: Vec<String>,
}

impl ScanDiagnostic {
    /// Returns the diagnostic message template.
    #[must_use]
    pub const fn message(&self) -> Message {
        self.message
    }

    /// Returns the UTF-8 byte offset where the diagnostic starts.
    #[must_use]
    pub const fn start(&self) -> usize {
        self.start
    }

    /// Returns the UTF-8 byte length of the diagnostic range.
    #[must_use]
    pub const fn length(&self) -> usize {
        self.length
    }

    /// Returns the formatted message text.
    #[must_use]
    pub fn text(&self) -> String {
        let arguments = self
            .arguments
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        self.message.format(&arguments)
    }
}

/// The restorable position and current-token state of a [`Scanner`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannerState {
    pos: usize,
    full_start_pos: usize,
    token_start: usize,
    token: SyntaxKind,
    token_value: String,
    token_flags: TokenFlags,
    diagnostic_count: usize,
}

/// Produces TypeScript tokens from source text on demand.
#[derive(Debug, Clone)]
pub struct Scanner<'text> {
    text: &'text str,
    skip_trivia: bool,
    state: ScannerState,
    diagnostics: Vec<ScanDiagnostic>,
}

impl<'text> Scanner<'text> {
    /// Creates a scanner positioned at the start of `text` that skips trivia.
    #[must_use]
    pub fn new(text: &'text str) -> Self {
        Self {
            text,
            skip_trivia: true,
            state: ScannerState {
                pos: 0,
                full_start_pos: 0,
                token_start: 0,
                token: SyntaxKind::Unknown,
                token_value: String::new(),
                token_flags: TokenFlags::NONE,
                diagnostic_count: 0,
            },
            diagnostics: Vec::new(),
        }
    }

    /// Returns a scanner that reports whitespace, newlines, and comments as tokens.
    #[must_use]
    pub const fn with_trivia(mut self) -> Self {
        self.skip_trivia = false;
        self
    }

    /// Returns the current token kind.
    #[must_use]
    pub const fn token(&self) -> SyntaxKind {
        self.state.token
    }

    /// Returns the flags of the current token.
    #[must_use]
    pub const fn token_flags(&self) -> TokenFlags {
        self.state.token_flags
    }

    /// Returns the start of the current token including preceding trivia.
    #[must_use]
    pub const fn token_full_start(&self) -> usize {
        self.state.full_start_pos
    }

    /// Returns the start of the current token excluding preceding trivia.
    #[must_use]
    pub const fn token_start(&self) -> usize {
        self.state.token_start
    }

    /// Returns the end of the current token.
    #[must_use]
    pub const fn token_end(&self) -> usize {
        self.state.pos
    }

    /// Returns the source text of the current token.
    #[must_use]
    pub fn token_text(&self) -> &'text str {
        &self.text[self.state.token_start..self.state.pos]
    }

    /// Returns the cooked value of the current identifier or literal token.
    #[must_use]
    pub fn token_value(&self) -> &str {
        &self.state.token_value
    }

    /// Returns whether a line break precedes the current token.
    #[must_use]
    pub const fn has_preceding_line_break(&self) -> bool {
        self.state
            .token_flags
            .intersects(TokenFlags::PRECEDING_LINE_BREAK)
    }

    /// Returns the diagnostics reported so far.
    #[must_use]
    pub fn diagnostics(&self) -> &[ScanDiagnostic] {
        &self.diagnostics
    }

    /// Captures the scanner state so that speculative scanning can be undone.
    #[must_use]
    pub fn mark(&self) -> ScannerState {
        ScannerState {
            diagnostic_count: self.diagnostics.len(),
            ..self.state.clone()
        }
    }

    /// Restores a state captured by [`Self::mark`], discarding later diagnostics.
    pub fn rewind(&mut self, state: ScannerState) {
        self.diagnostics.truncate(state.diagnostic_count);
        self.state = state;
    }

    /// Scans the next token and returns its kind.
    pub fn scan(&mut self) -> SyntaxKind {
        self.state.full_start_pos = self.state.pos;
        self.state.token_flags = TokenFlags::NONE;
        loop {
            self.state.token_start = self.state.pos;
            let Some(byte) = self.byte_at(0) else {
                return self.finish(SyntaxKind::EndOfFile, 0);
            };
            let token = match byte {
                b'\t' | 0x0b | 0x0c | b' ' => {
                    self.state.pos += 1;
                    if self.skip_trivia {
                        continue;
                    }
                    self.skip_single_line_white_space();
                    SyntaxKind::WhitespaceTrivia
                }
                b'\n' | b'\r' => {
                    self.state.token_flags |= TokenFlags::PRECEDING_LINE_BREAK;
                    if self.skip_trivia {
                        self.state.pos += 1;
                        continue;
                    }
                    let length = if byte == b'\r' && self.byte_at(1) == Some(b'\n') {
                        2
                    } else {
                        1
                    };
                    self.state.pos += length;
                    SyntaxKind::NewLineTrivia
                }
                b'/' if self.byte_at(1) == Some(b'/') => {
                    self.scan_single_line_comment();
                    if self.skip_trivia {
                        continue;
                    }
                    SyntaxKind::SingleLineCommentTrivia
                }
                b'/' if self.byte_at(1) == Some(b'*') => {
                    self.scan_multi_line_comment();
                    if self.skip_trivia {
                        continue;
                    }
                    SyntaxKind::MultiLineCommentTrivia
                }
                _ => match self.scan_other_token(byte) {
                    Some(token) => token,
                    None => continue,
                },
            };
            self.state.token = token;
            return token;
        }
    }

    fn finish(&mut self, token: SyntaxKind, length: usize) -> SyntaxKind {
        self.state.pos += length;
        self.state.token = token;
        token
    }

    fn byte_at(&self, offset: usize) -> Option<u8> {
        self.text.as_bytes().get(self.state.pos + offset).copied()
    }

    fn char_at_pos(&self) -> Option<char> {
        self.text[self.state.pos..].chars().next()
    }

    fn error(&mut self, message: Message) {
        self.error_at(message, self.state.pos, 0, &[]);
    }

    fn error_at(&mut self, message: Message, start: usize, length: usize, arguments: &[&str]) {
        self.diagnostics.push(ScanDiagnostic {
            message,
            start,
            length,
            arguments: arguments
                .iter()
                .map(|&argument| argument.to_owned())
                .collect(),
        });
    }

    fn skip_single_line_white_space(&mut self) {
        while let Some(character) = self.char_at_pos() {
            if !is_white_space_single_line(character) {
                break;
            }
            self.state.pos += character.len_utf8();
        }
    }

    fn scan_single_line_comment(&mut self) {
        self.state.pos += 2;
        while let Some(character) = self.char_at_pos() {
            if is_line_break(character) {
                break;
            }
            self.state.pos += character.len_utf8();
        }
    }

    fn scan_multi_line_comment(&mut self) {
        self.state.pos += 2;
        let is_jsdoc = self.byte_at(0) == Some(b'*') && self.byte_at(1) != Some(b'/');
        let mut closed = false;
        while let Some(character) = self.char_at_pos() {
            if character == '*' && self.byte_at(1) == Some(b'/') {
                self.state.pos += 2;
                closed = true;
                break;
            }
            self.state.pos += character.len_utf8();
            if is_line_break(character) {
                self.state.token_flags |= TokenFlags::PRECEDING_LINE_BREAK;
            }
        }
        if is_jsdoc {
            self.state.token_flags |= TokenFlags::PRECEDING_JSDOC_COMMENT;
        }
        if !closed {
            self.error(diagnostics::ASTERISK_SLASH_EXPECTED);
            if !self.skip_trivia {
                self.state.token_flags |= TokenFlags::UNTERMINATED;
            }
        }
    }

    /// Scans a token that is not ASCII trivia. Returns `None` when trivia was skipped.
    fn scan_other_token(&mut self, byte: u8) -> Option<SyntaxKind> {
        if matches!(byte, b'"' | b'\'') {
            self.state.token_value = self.scan_string(false);
            return Some(SyntaxKind::StringLiteral);
        }
        if let Some(token) = self.scan_punctuation(byte) {
            return Some(token);
        }
        if self.scan_identifier(0) {
            return Some(identifier_token(&self.state.token_value));
        }
        match self.scan_non_ascii_trivia() {
            Some(trivia) => trivia,
            None => {
                self.scan_invalid_character();
                Some(SyntaxKind::Unknown)
            }
        }
    }

    /// Scans an identifier after `prefix_length` bytes and stores its text as the token value.
    fn scan_identifier(&mut self, prefix_length: usize) -> bool {
        let start = self.state.pos;
        self.state.pos += prefix_length;
        match self.char_at_pos() {
            Some(character) if is_identifier_start(character) => {
                self.state.pos += character.len_utf8();
            }
            _ => {
                self.state.pos = start;
                return false;
            }
        }
        while let Some(character) = self.char_at_pos() {
            if !is_identifier_part(character) {
                break;
            }
            self.state.pos += character.len_utf8();
        }
        self.state.token_value = self.text[start..self.state.pos].to_owned();
        true
    }

    /// Scans non-ASCII whitespace and line breaks. Returns `Some(None)` when trivia was skipped.
    fn scan_non_ascii_trivia(&mut self) -> Option<Option<SyntaxKind>> {
        let character = self.char_at_pos()?;
        if is_white_space_single_line(character) {
            self.state.pos += character.len_utf8();
            if character == '\u{85}' || self.skip_trivia {
                return Some(None);
            }
            self.skip_single_line_white_space();
            return Some(Some(SyntaxKind::WhitespaceTrivia));
        }
        if is_line_break(character) {
            self.state.token_flags |= TokenFlags::PRECEDING_LINE_BREAK;
            self.state.pos += character.len_utf8();
            return Some(None);
        }
        None
    }

    fn scan_invalid_character(&mut self) {
        let length = self.char_at_pos().map_or(1, char::len_utf8);
        self.error_at(diagnostics::INVALID_CHARACTER, self.state.pos, length, &[]);
        self.state.pos += length;
    }

    fn scan_punctuation(&mut self, byte: u8) -> Option<SyntaxKind> {
        let next = self.byte_at(1);
        let after_next = self.byte_at(2);
        let (token, length) = match (byte, next, after_next) {
            (b'!', Some(b'='), Some(b'=')) => (SyntaxKind::ExclamationEqualsEqualsToken, 3),
            (b'!', Some(b'='), _) => (SyntaxKind::ExclamationEqualsToken, 2),
            (b'!', _, _) => (SyntaxKind::ExclamationToken, 1),
            (b'%', Some(b'='), _) => (SyntaxKind::PercentEqualsToken, 2),
            (b'%', _, _) => (SyntaxKind::PercentToken, 1),
            (b'&', Some(b'&'), Some(b'=')) => (SyntaxKind::AmpersandAmpersandEqualsToken, 3),
            (b'&', Some(b'&'), _) => (SyntaxKind::AmpersandAmpersandToken, 2),
            (b'&', Some(b'='), _) => (SyntaxKind::AmpersandEqualsToken, 2),
            (b'&', _, _) => (SyntaxKind::AmpersandToken, 1),
            (b'(', _, _) => (SyntaxKind::OpenParenToken, 1),
            (b')', _, _) => (SyntaxKind::CloseParenToken, 1),
            (b'*', Some(b'='), _) => (SyntaxKind::AsteriskEqualsToken, 2),
            (b'*', Some(b'*'), Some(b'=')) => (SyntaxKind::AsteriskAsteriskEqualsToken, 3),
            (b'*', Some(b'*'), _) => (SyntaxKind::AsteriskAsteriskToken, 2),
            (b'*', _, _) => (SyntaxKind::AsteriskToken, 1),
            (b'+', Some(b'='), _) => (SyntaxKind::PlusEqualsToken, 2),
            (b'+', Some(b'+'), _) => (SyntaxKind::PlusPlusToken, 2),
            (b'+', _, _) => (SyntaxKind::PlusToken, 1),
            (b',', _, _) => (SyntaxKind::CommaToken, 1),
            (b'-', Some(b'='), _) => (SyntaxKind::MinusEqualsToken, 2),
            (b'-', Some(b'-'), _) => (SyntaxKind::MinusMinusToken, 2),
            (b'-', _, _) => (SyntaxKind::MinusToken, 1),
            (b'.', Some(b'.'), Some(b'.')) => (SyntaxKind::DotDotDotToken, 3),
            (b'.', Some(digit), _) if digit.is_ascii_digit() => return None,
            (b'.', _, _) => (SyntaxKind::DotToken, 1),
            (b'/', Some(b'='), _) => (SyntaxKind::SlashEqualsToken, 2),
            (b'/', _, _) => (SyntaxKind::SlashToken, 1),
            (b':', _, _) => (SyntaxKind::ColonToken, 1),
            (b';', _, _) => (SyntaxKind::SemicolonToken, 1),
            (b'<', Some(b'<'), Some(b'=')) => (SyntaxKind::LessThanLessThanEqualsToken, 3),
            (b'<', Some(b'<'), _) => (SyntaxKind::LessThanLessThanToken, 2),
            (b'<', Some(b'='), _) => (SyntaxKind::LessThanEqualsToken, 2),
            (b'<', _, _) => (SyntaxKind::LessThanToken, 1),
            (b'=', Some(b'='), Some(b'=')) => (SyntaxKind::EqualsEqualsEqualsToken, 3),
            (b'=', Some(b'='), _) => (SyntaxKind::EqualsEqualsToken, 2),
            (b'=', Some(b'>'), _) => (SyntaxKind::EqualsGreaterThanToken, 2),
            (b'=', _, _) => (SyntaxKind::EqualsToken, 1),
            (b'>', _, _) => (SyntaxKind::GreaterThanToken, 1),
            (b'?', Some(b'.'), digit) if !digit.is_some_and(|digit| digit.is_ascii_digit()) => {
                (SyntaxKind::QuestionDotToken, 2)
            }
            (b'?', Some(b'?'), Some(b'=')) => (SyntaxKind::QuestionQuestionEqualsToken, 3),
            (b'?', Some(b'?'), _) => (SyntaxKind::QuestionQuestionToken, 2),
            (b'?', _, _) => (SyntaxKind::QuestionToken, 1),
            (b'[', _, _) => (SyntaxKind::OpenBracketToken, 1),
            (b']', _, _) => (SyntaxKind::CloseBracketToken, 1),
            (b'^', Some(b'='), _) => (SyntaxKind::CaretEqualsToken, 2),
            (b'^', _, _) => (SyntaxKind::CaretToken, 1),
            (b'{', _, _) => (SyntaxKind::OpenBraceToken, 1),
            (b'|', Some(b'|'), Some(b'=')) => (SyntaxKind::BarBarEqualsToken, 3),
            (b'|', Some(b'|'), _) => (SyntaxKind::BarBarToken, 2),
            (b'|', Some(b'='), _) => (SyntaxKind::BarEqualsToken, 2),
            (b'|', _, _) => (SyntaxKind::BarToken, 1),
            (b'}', _, _) => (SyntaxKind::CloseBraceToken, 1),
            (b'~', _, _) => (SyntaxKind::TildeToken, 1),
            (b'@', _, _) => (SyntaxKind::AtToken, 1),
            _ => return None,
        };
        self.state.pos += length;
        Some(token)
    }
}
