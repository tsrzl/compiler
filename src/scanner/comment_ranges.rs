//! Comment ranges around a position, modeled on TypeScript-Go's `iterateCommentRanges`.

use super::chars::{is_line_break, is_white_space_like};
use super::trivia::scan_shebang_trivia;
use crate::ast::SyntaxKind;

/// A comment and whether a line break follows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentRange {
    /// `SingleLineCommentTrivia` or `MultiLineCommentTrivia`.
    pub kind: SyntaxKind,
    /// The UTF-8 byte offset of the comment's first character.
    pub pos: usize,
    /// The UTF-8 byte offset just past the comment; single-line comments exclude the line break.
    pub end: usize,
    /// Whether a line break follows the comment.
    pub has_trailing_new_line: bool,
}

/// Returns the comments before the next token at or after `pos`. At a line start, and at the
/// start of the file after any shebang, every comment is collected; otherwise only comments
/// after the first line break are.
#[must_use]
pub fn leading_comment_ranges(text: &str, pos: usize) -> Vec<CommentRange> {
    comment_ranges(text, pos, false)
}

/// Returns the comments after `pos` up to the end of its line.
#[must_use]
pub fn trailing_comment_ranges(text: &str, pos: usize) -> Vec<CommentRange> {
    comment_ranges(text, pos, true)
}

fn comment_ranges(text: &str, mut pos: usize, trailing: bool) -> Vec<CommentRange> {
    let bytes = text.as_bytes();
    let mut ranges = Vec::new();
    let mut pending: Option<CommentRange> = None;
    let mut collecting = trailing;
    if pos == 0 {
        collecting = true;
        if text.starts_with("#!") {
            pos = scan_shebang_trivia(text, pos);
        }
    }
    while let Some(character) = text.get(pos..).and_then(|rest| rest.chars().next()) {
        match character {
            '\r' | '\n' => {
                if character == '\r' && bytes.get(pos + 1) == Some(&b'\n') {
                    pos += 1;
                }
                pos += 1;
                if trailing {
                    break;
                }
                collecting = true;
                if let Some(pending) = pending.as_mut() {
                    pending.has_trailing_new_line = true;
                }
            }
            '\t' | '\u{000B}' | '\u{000C}' | ' ' => pos += 1,
            '/' if matches!(bytes.get(pos + 1), Some(b'/' | b'*')) => {
                let single_line = bytes[pos + 1] == b'/';
                let start = pos;
                pos += 2;
                let mut has_trailing_new_line = false;
                if single_line {
                    match text[pos..].find(is_line_break) {
                        Some(offset) => {
                            pos += offset;
                            has_trailing_new_line = true;
                        }
                        None => pos = text.len(),
                    }
                } else {
                    pos = text[pos..]
                        .find("*/")
                        .map_or(text.len(), |offset| pos + offset + 2);
                }
                if collecting {
                    ranges.extend(pending.take());
                    pending = Some(CommentRange {
                        kind: if single_line {
                            SyntaxKind::SingleLineCommentTrivia
                        } else {
                            SyntaxKind::MultiLineCommentTrivia
                        },
                        pos: start,
                        end: pos,
                        has_trailing_new_line,
                    });
                }
            }
            _ if !character.is_ascii() && is_white_space_like(character) => {
                if is_line_break(character)
                    && let Some(pending) = pending.as_mut()
                {
                    pending.has_trailing_new_line = true;
                }
                pos += character.len_utf8();
            }
            _ => break,
        }
    }
    ranges.extend(pending);
    ranges
}
