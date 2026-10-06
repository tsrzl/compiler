//! The scanner cursor and current-token state, owned behind a narrow API.

use crate::ast::{SyntaxKind, TokenFlags};

/// The restorable position and current-token state of a [`super::Scanner`].
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

impl ScannerState {
    pub(super) const fn new() -> Self {
        Self {
            pos: 0,
            full_start_pos: 0,
            token_start: 0,
            token: SyntaxKind::Unknown,
            token_value: String::new(),
            token_flags: TokenFlags::NONE,
            diagnostic_count: 0,
        }
    }

    /// Starts a new token at the current position, clearing its flags.
    pub(super) fn begin_token(&mut self) {
        self.full_start_pos = self.pos;
        self.token_flags = TokenFlags::NONE;
    }

    /// Marks the current position as the start of the token's non-trivia text.
    pub(super) fn begin_token_text(&mut self) {
        self.token_start = self.pos;
    }

    pub(super) fn finish_token(&mut self, token: SyntaxKind) {
        self.token = token;
    }

    pub(super) const fn pos(&self) -> usize {
        self.pos
    }

    pub(super) fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    pub(super) fn advance(&mut self, length: usize) {
        self.pos += length;
    }

    pub(super) fn retreat(&mut self, length: usize) {
        self.pos -= length;
    }

    pub(super) const fn full_start(&self) -> usize {
        self.full_start_pos
    }

    pub(super) const fn token_start(&self) -> usize {
        self.token_start
    }

    pub(super) const fn token(&self) -> SyntaxKind {
        self.token
    }

    pub(super) fn token_value(&self) -> &str {
        &self.token_value
    }

    pub(super) fn set_token_value(&mut self, value: String) {
        self.token_value = value;
    }

    pub(super) fn push_token_value(&mut self, suffix: char) {
        self.token_value.push(suffix);
    }

    pub(super) const fn flags(&self) -> TokenFlags {
        self.token_flags
    }

    pub(super) fn set_flags(&mut self, flags: TokenFlags) {
        self.token_flags = flags;
    }

    pub(super) fn add_flags(&mut self, flags: TokenFlags) {
        self.token_flags |= flags;
    }

    pub(super) const fn diagnostic_count(&self) -> usize {
        self.diagnostic_count
    }

    /// Returns a copy that records how many diagnostics existed when it was captured.
    pub(super) fn snapshot(&self, diagnostic_count: usize) -> Self {
        Self {
            diagnostic_count,
            ..self.clone()
        }
    }
}
