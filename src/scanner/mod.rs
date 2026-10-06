//! The TypeScript scanner, ported from TypeScript-Go's `internal/scanner` package.
//!
//! Positions are UTF-8 byte offsets into the source text, matching TypeScript-Go. Convert them to
//! UTF-16 offsets with [`crate::source_text::SourceText`] at reporting boundaries.

mod chars;
mod identifier_tables;
mod identifiers;
mod keywords;
mod numbers;
mod punctuation;
mod state;
mod strings;
mod templates;
mod trivia;

use crate::ast::{SyntaxKind, TokenFlags};
use crate::diagnostics::{self, Message};

pub use keywords::{identifier_token, keyword};
pub use state::ScannerState;
pub use trivia::{CommentDirective, CommentDirectiveKind};

/// Whether the scanner recognizes JSX-specific tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LanguageVariant {
    /// Standard TypeScript or JavaScript.
    #[default]
    Standard,
    /// TypeScript or JavaScript with JSX.
    Jsx,
}

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

/// Produces TypeScript tokens from source text on demand.
#[derive(Debug, Clone)]
pub struct Scanner<'text> {
    text: &'text str,
    skip_trivia: bool,
    language_variant: LanguageVariant,
    state: ScannerState,
    diagnostics: Vec<ScanDiagnostic>,
    comment_directives: Vec<CommentDirective>,
}

impl<'text> Scanner<'text> {
    /// Creates a scanner positioned at the start of `text` that skips trivia.
    #[must_use]
    pub fn new(text: &'text str) -> Self {
        Self {
            text,
            skip_trivia: true,
            language_variant: LanguageVariant::Standard,
            state: ScannerState::new(),
            diagnostics: Vec::new(),
            comment_directives: Vec::new(),
        }
    }

    /// Returns a scanner that recognizes tokens for `language_variant`.
    #[must_use]
    pub const fn with_language_variant(mut self, language_variant: LanguageVariant) -> Self {
        self.language_variant = language_variant;
        self
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
        self.state.token()
    }

    /// Returns the flags of the current token.
    #[must_use]
    pub const fn token_flags(&self) -> TokenFlags {
        self.state.flags()
    }

    /// Returns the start of the current token including preceding trivia.
    #[must_use]
    pub const fn token_full_start(&self) -> usize {
        self.state.full_start()
    }

    /// Returns the start of the current token excluding preceding trivia.
    #[must_use]
    pub const fn token_start(&self) -> usize {
        self.state.token_start()
    }

    /// Returns the end of the current token.
    #[must_use]
    pub const fn token_end(&self) -> usize {
        self.state.pos()
    }

    /// Returns the source text of the current token.
    #[must_use]
    pub fn token_text(&self) -> &'text str {
        &self.text[self.state.token_start()..self.state.pos()]
    }

    /// Returns the cooked value of the current identifier or literal token.
    #[must_use]
    pub fn token_value(&self) -> &str {
        self.state.token_value()
    }

    /// Returns whether a line break precedes the current token.
    #[must_use]
    pub const fn has_preceding_line_break(&self) -> bool {
        self.state
            .flags()
            .intersects(TokenFlags::PRECEDING_LINE_BREAK)
    }

    /// Returns the diagnostics reported so far.
    #[must_use]
    pub fn diagnostics(&self) -> &[ScanDiagnostic] {
        &self.diagnostics
    }

    /// Returns the `@ts-expect-error` and `@ts-ignore` comments scanned so far.
    #[must_use]
    pub fn comment_directives(&self) -> &[CommentDirective] {
        &self.comment_directives
    }

    /// Captures the scanner state so that speculative scanning can be undone.
    #[must_use]
    pub fn mark(&self) -> ScannerState {
        self.state.snapshot(self.diagnostics.len())
    }

    /// Restores a state captured by [`Self::mark`], discarding later diagnostics.
    pub fn rewind(&mut self, state: ScannerState) {
        self.diagnostics.truncate(state.diagnostic_count());
        self.state = state;
    }

    /// Scans the next token and returns its kind.
    pub fn scan(&mut self) -> SyntaxKind {
        self.state.begin_token();
        loop {
            self.state.begin_token_text();
            if let Some(token) = self.scan_step() {
                self.state.finish_token(token);
                return token;
            }
        }
    }

    /// Scans one token, returning `None` when trivia was skipped.
    fn scan_step(&mut self) -> Option<SyntaxKind> {
        let Some(byte) = self.byte_at(0) else {
            return Some(SyntaxKind::EndOfFile);
        };
        match byte {
            b'\t' | 0x0b | 0x0c | b' ' => {
                self.state.advance(1);
                if self.skip_trivia {
                    return None;
                }
                self.skip_single_line_white_space();
                Some(SyntaxKind::WhitespaceTrivia)
            }
            b'\n' | b'\r' => {
                self.state.add_flags(TokenFlags::PRECEDING_LINE_BREAK);
                let length = if byte == b'\r' && self.byte_at(1) == Some(b'\n') {
                    2
                } else {
                    1
                };
                self.state.advance(length);
                (!self.skip_trivia).then_some(SyntaxKind::NewLineTrivia)
            }
            b'/' if self.byte_at(1) == Some(b'/') => {
                self.scan_single_line_comment();
                (!self.skip_trivia).then_some(SyntaxKind::SingleLineCommentTrivia)
            }
            b'/' if self.byte_at(1) == Some(b'*') => {
                self.scan_multi_line_comment();
                (!self.skip_trivia).then_some(SyntaxKind::MultiLineCommentTrivia)
            }
            b'<' | b'=' | b'>' | b'|' => match self.scan_conflict_marker() {
                Some(trivia) => trivia,
                None => self.scan_punctuation(byte),
            },
            b'"' | b'\'' => {
                let value = self.scan_string(false);
                self.state.set_token_value(value);
                Some(SyntaxKind::StringLiteral)
            }
            b'`' => Some(self.scan_template_and_set_token_value(false)),
            b'#' => self.scan_hash(),
            b'\\' => Some(self.scan_escaped_identifier()),
            b'0' if matches!(
                self.byte_at(1),
                Some(b'x' | b'X' | b'b' | b'B' | b'o' | b'O')
            ) =>
            {
                self.scan_prefixed_number()
            }
            b'0'..=b'9' => Some(self.scan_number()),
            b'.' if self.byte_at(1).is_some_and(|next| next.is_ascii_digit()) => {
                Some(self.scan_number())
            }
            _ => self
                .scan_punctuation(byte)
                .or_else(|| self.scan_word_or_other()),
        }
    }

    /// Scans an identifier, non-ASCII trivia, or an invalid character.
    fn scan_word_or_other(&mut self) -> Option<SyntaxKind> {
        if self.scan_identifier(0) {
            return Some(identifier_token(self.state.token_value()));
        }
        // TypeScript-Go decodes U+FFFD as its UTF-8 error rune and treats the file as binary.
        if self.char_at_pos() == Some(char::REPLACEMENT_CHARACTER) {
            self.error_at(diagnostics::FILE_APPEARS_TO_BE_BINARY, 0, 0, &[]);
            self.state.set_pos(self.text.len());
            return Some(SyntaxKind::NonTextFileMarkerTrivia);
        }
        match self.scan_non_ascii_trivia() {
            Some(trivia) => trivia,
            None => {
                self.scan_invalid_character();
                Some(SyntaxKind::Unknown)
            }
        }
    }

    fn byte_at(&self, offset: usize) -> Option<u8> {
        self.text.as_bytes().get(self.state.pos() + offset).copied()
    }

    fn char_at_pos(&self) -> Option<char> {
        self.text[self.state.pos()..].chars().next()
    }

    fn error(&mut self, message: Message) {
        self.error_at(message, self.state.pos(), 0, &[]);
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

    fn scan_invalid_character(&mut self) {
        let length = self.char_at_pos().map_or(1, char::len_utf8);
        self.error_at(
            diagnostics::INVALID_CHARACTER,
            self.state.pos(),
            length,
            &[],
        );
        self.state.advance(length);
    }
}
