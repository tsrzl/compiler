//! Whitespace, comment, comment-directive, conflict-marker, and shebang scanning.

use crate::ast::{SyntaxKind, TokenFlags};
use crate::diagnostics;

use super::Scanner;
use super::chars::{is_line_break, is_white_space_single_line};

const MERGE_CONFLICT_MARKER_LENGTH: usize = 7;

/// The suppression requested by a `@ts-expect-error` or `@ts-ignore` comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommentDirectiveKind {
    /// A `@ts-expect-error` comment.
    ExpectError,
    /// A `@ts-ignore` comment.
    Ignore,
}

/// A suppression comment and the UTF-8 byte range it occupies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommentDirective {
    kind: CommentDirectiveKind,
    start: usize,
    end: usize,
}

impl CommentDirective {
    /// Returns the requested suppression.
    #[must_use]
    pub const fn kind(self) -> CommentDirectiveKind {
        self.kind
    }

    /// Returns the UTF-8 byte offset where the directive range starts.
    #[must_use]
    pub const fn start(self) -> usize {
        self.start
    }

    /// Returns the UTF-8 byte offset where the directive range ends.
    #[must_use]
    pub const fn end(self) -> usize {
        self.end
    }
}

impl Scanner<'_> {
    pub(super) fn skip_single_line_white_space(&mut self) {
        while let Some(character) = self.char_at_pos() {
            if !is_white_space_single_line(character) {
                break;
            }
            self.state.advance(character.len_utf8());
        }
    }

    pub(super) fn scan_single_line_comment(&mut self) {
        self.state.advance(2);
        while let Some(character) = self.char_at_pos() {
            if is_line_break(character) {
                break;
            }
            self.state.advance(character.len_utf8());
        }
        self.process_comment_directive(self.state.token_start(), self.state.pos(), false);
    }

    pub(super) fn scan_multi_line_comment(&mut self) {
        self.state.advance(2);
        let is_jsdoc = self.byte_at(0) == Some(b'*') && self.byte_at(1) != Some(b'/');
        let mut closed = false;
        let mut last_line_start = self.state.token_start();
        while let Some(character) = self.char_at_pos() {
            if character == '*' && self.byte_at(1) == Some(b'/') {
                self.state.advance(2);
                closed = true;
                break;
            }
            self.state.advance(character.len_utf8());
            if is_line_break(character) {
                last_line_start = self.state.pos();
                self.state.add_flags(TokenFlags::PRECEDING_LINE_BREAK);
            }
        }
        if is_jsdoc {
            self.state.add_flags(TokenFlags::PRECEDING_JSDOC_COMMENT);
            self.scan_jsdoc_comment_for_tags(self.state.token_start(), self.state.pos());
        }
        self.process_comment_directive(last_line_start, self.state.pos(), true);
        if !closed {
            self.error(diagnostics::ASTERISK_SLASH_EXPECTED);
            if !self.skip_trivia {
                self.state.add_flags(TokenFlags::UNTERMINATED);
            }
        }
    }

    /// Scans non-ASCII whitespace and line breaks. Returns `Some(None)` when trivia was skipped.
    pub(super) fn scan_non_ascii_trivia(&mut self) -> Option<Option<SyntaxKind>> {
        let character = self.char_at_pos()?;
        if is_white_space_single_line(character) {
            self.state.advance(character.len_utf8());
            if character == '\u{85}' || self.skip_trivia {
                return Some(None);
            }
            self.skip_single_line_white_space();
            return Some(Some(SyntaxKind::WhitespaceTrivia));
        }
        if is_line_break(character) {
            self.state.add_flags(TokenFlags::PRECEDING_LINE_BREAK);
            self.state.advance(character.len_utf8());
            return Some(None);
        }
        None
    }

    /// Scans a merge conflict marker at the current position, if one starts here.
    ///
    /// Returns `Some(None)` when the marker was skipped as trivia.
    pub(super) fn scan_conflict_marker(&mut self) -> Option<Option<SyntaxKind>> {
        if !is_conflict_marker_trivia(self.text, self.state.pos()) {
            return None;
        }
        self.skip_conflict_marker();
        Some((!self.skip_trivia).then_some(SyntaxKind::ConflictMarkerTrivia))
    }

    /// Reports and skips the merge conflict marker at the current position.
    pub(super) fn skip_conflict_marker(&mut self) {
        self.error_at(
            diagnostics::MERGE_CONFLICT_MARKER_ENCOUNTERED,
            self.state.pos(),
            MERGE_CONFLICT_MARKER_LENGTH,
            &[],
        );
        self.state
            .set_pos(scan_conflict_marker_trivia(self.text, self.state.pos()));
    }

    fn process_comment_directive(&mut self, start: usize, end: usize, multiline: bool) {
        let bytes = self.text.as_bytes();
        let mut pos = start;
        if multiline {
            while pos < end && matches!(bytes[pos], b' ' | b'\t') {
                pos += 1;
            }
            while pos < end && matches!(bytes[pos], b'/' | b'*') {
                pos += 1;
            }
        } else {
            pos += 2;
            while pos < end && bytes[pos] == b'/' {
                pos += 1;
            }
        }
        while pos < end && matches!(bytes[pos], b' ' | b'\t') {
            pos += 1;
        }
        if pos >= end || bytes[pos] != b'@' {
            return;
        }
        let directive = &self.text[pos + 1..];
        let kind = if directive.starts_with("ts-expect-error") {
            CommentDirectiveKind::ExpectError
        } else if directive.starts_with("ts-ignore") {
            CommentDirectiveKind::Ignore
        } else {
            return;
        };
        self.comment_directives
            .push(CommentDirective { kind, start, end });
    }

    fn scan_jsdoc_comment_for_tags(&mut self, start: usize, end: usize) {
        let mut comment = &self.text[start..end];
        while let Some(at) = comment.find('@') {
            comment = &comment[at + 1..];
            if has_jsdoc_tag(comment, &["deprecated"]) {
                self.state
                    .add_flags(TokenFlags::PRECEDING_JSDOC_WITH_DEPRECATED);
            }
            if has_jsdoc_tag(comment, &["see", "link", "linkcode", "linkplain"]) {
                self.state
                    .add_flags(TokenFlags::PRECEDING_JSDOC_WITH_SEE_OR_LINK);
            }
            let found = self.state.flags()
                & (TokenFlags::PRECEDING_JSDOC_WITH_DEPRECATED
                    | TokenFlags::PRECEDING_JSDOC_WITH_SEE_OR_LINK);
            if found
                == TokenFlags::PRECEDING_JSDOC_WITH_DEPRECATED
                    | TokenFlags::PRECEDING_JSDOC_WITH_SEE_OR_LINK
            {
                return;
            }
        }
    }
}

fn has_jsdoc_tag(text: &str, tags: &[&str]) -> bool {
    tags.iter().any(|tag| {
        text.strip_prefix(tag).is_some_and(|rest| {
            rest.bytes()
                .next()
                .is_none_or(|byte| matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b'}' | b'*'))
        })
    })
}

/// Returns whether a merge conflict marker starts at `pos`, following TypeScript-Go exactly.
pub(super) fn is_conflict_marker_trivia(text: &str, pos: usize) -> bool {
    let bytes = text.as_bytes();
    if pos + 1 >= bytes.len() || bytes[pos + 1] != bytes[pos] {
        return false;
    }
    let mut at_line_start = pos == 0 || is_line_break(char::from(bytes[pos - 1]));
    if !at_line_start && pos >= 2 {
        // TypeScript-Go inspects the character ending before `pos - 2` here; match it exactly.
        at_line_start = text
            .get(..pos - 2)
            .and_then(|prefix| prefix.chars().next_back())
            .is_some_and(is_line_break);
    }
    if !at_line_start || pos + MERGE_CONFLICT_MARKER_LENGTH >= bytes.len() {
        return false;
    }
    let marker = bytes[pos];
    bytes[pos..pos + MERGE_CONFLICT_MARKER_LENGTH]
        .iter()
        .all(|&byte| byte == marker)
        && (marker == b'=' || bytes[pos + MERGE_CONFLICT_MARKER_LENGTH] == b' ')
}

fn scan_conflict_marker_trivia(text: &str, mut pos: usize) -> usize {
    let bytes = text.as_bytes();
    let marker = bytes[pos];
    if matches!(marker, b'<' | b'>') {
        while let Some(character) = text[pos..].chars().next() {
            if is_line_break(character) {
                break;
            }
            pos += character.len_utf8();
        }
        return pos;
    }
    while pos < bytes.len() {
        let current = bytes[pos];
        if matches!(current, b'=' | b'>')
            && current != marker
            && is_conflict_marker_trivia(text, pos)
        {
            break;
        }
        pos += 1;
    }
    pos
}

/// Returns the end of a `#!` shebang line starting at `pos`.
pub(super) fn scan_shebang_trivia(text: &str, pos: usize) -> usize {
    let mut pos = pos + 2;
    while let Some(character) = text[pos..].chars().next() {
        if is_line_break(character) {
            break;
        }
        pos += character.len_utf8();
    }
    pos
}

/// Options for [`skip_trivia_with`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SkipTriviaOptions {
    /// Stop immediately after the first line break.
    pub stop_after_line_break: bool,
    /// Stop at the start of a comment instead of skipping it.
    pub stop_at_comments: bool,
    /// Skip a leading `*` after each line break, as at the start of JSDoc lines.
    pub in_jsdoc: bool,
}

/// Returns the position of the first non-trivia character at or after `pos`.
#[must_use]
pub fn skip_trivia(text: &str, pos: usize) -> usize {
    skip_trivia_with(text, pos, SkipTriviaOptions::default())
}

/// Returns the position of the first non-trivia character at or after `pos`, using `options`.
#[must_use]
pub fn skip_trivia_with(text: &str, mut pos: usize, options: SkipTriviaOptions) -> usize {
    let bytes = text.as_bytes();
    let mut can_consume_star = false;
    while let Some(character) = text.get(pos..).and_then(|rest| rest.chars().next()) {
        match character {
            '\r' | '\n' => {
                if character == '\r' && bytes.get(pos + 1) == Some(&b'\n') {
                    pos += 1;
                }
                pos += 1;
                if options.stop_after_line_break {
                    return pos;
                }
                can_consume_star = options.in_jsdoc;
            }
            '\t' | '\u{b}' | '\u{c}' | ' ' => pos += 1,
            '/' if !options.stop_at_comments && bytes.get(pos + 1) == Some(&b'/') => {
                pos += 2;
                while let Some(next) = text[pos..].chars().next() {
                    if is_line_break(next) {
                        break;
                    }
                    pos += next.len_utf8();
                }
                can_consume_star = false;
            }
            '/' if !options.stop_at_comments && bytes.get(pos + 1) == Some(&b'*') => {
                pos += 2;
                while pos < bytes.len() {
                    if bytes[pos] == b'*' && bytes.get(pos + 1) == Some(&b'/') {
                        pos += 2;
                        break;
                    }
                    pos += text[pos..].chars().next().map_or(1, char::len_utf8);
                }
                can_consume_star = false;
            }
            '<' | '|' | '=' | '>' if is_conflict_marker_trivia(text, pos) => {
                pos = scan_conflict_marker_trivia(text, pos);
                can_consume_star = false;
            }
            '#' if pos == 0 && bytes.get(1) == Some(&b'!') => {
                pos = scan_shebang_trivia(text, pos);
                can_consume_star = false;
            }
            '*' if can_consume_star => {
                pos += 1;
                can_consume_star = false;
            }
            _ if !character.is_ascii() && super::chars::is_white_space_like(character) => {
                pos += character.len_utf8();
            }
            _ => return pos,
        }
    }
    pos
}
