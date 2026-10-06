//! Context-sensitive rescanning of the current token, driven by the parser.

use crate::ast::{SyntaxKind, TokenFlags};
use crate::diagnostics;

use super::Scanner;
use super::chars::{is_identifier_part, is_line_break, is_white_space_like};

impl Scanner<'_> {
    /// Splits a `<<` token into `<` for type argument lists.
    pub fn rescan_less_than_token(&mut self) -> SyntaxKind {
        if self.state.token() == SyntaxKind::LessThanLessThanToken {
            self.rescan_as(SyntaxKind::LessThanToken, 1)
        } else {
            self.state.token()
        }
    }

    /// Combines `>` with following `>` and `=` characters into a shift or comparison token.
    pub fn rescan_greater_than_token(&mut self) -> SyntaxKind {
        if self.state.token() != SyntaxKind::GreaterThanToken {
            return self.state.token();
        }
        self.state.set_pos(self.state.token_start() + 1);
        let (token, length) = match (self.byte_at(0), self.byte_at(1), self.byte_at(2)) {
            (Some(b'>'), Some(b'>'), Some(b'=')) => {
                (SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken, 3)
            }
            (Some(b'>'), Some(b'>'), _) => (SyntaxKind::GreaterThanGreaterThanGreaterThanToken, 2),
            (Some(b'>'), Some(b'='), _) => (SyntaxKind::GreaterThanGreaterThanEqualsToken, 2),
            (Some(b'>'), _, _) => (SyntaxKind::GreaterThanGreaterThanToken, 1),
            (Some(b'='), _, _) => (SyntaxKind::GreaterThanEqualsToken, 1),
            _ => return self.state.token(),
        };
        self.state.advance(length);
        self.state.finish_token(token);
        token
    }

    /// Rescans a `}` as the continuation of a template literal.
    pub fn rescan_template_token(&mut self, is_tagged_template: bool) -> SyntaxKind {
        self.state.set_pos(self.state.token_start());
        let token = self.scan_template_and_set_token_value(!is_tagged_template);
        self.state.finish_token(token);
        token
    }

    /// Splits `*=` into `*` and rescans as `=`, as after a generator's `*` in a default.
    ///
    /// # Panics
    ///
    /// Panics when the current token is not `*=`.
    pub fn rescan_asterisk_equals_token(&mut self) -> SyntaxKind {
        assert_eq!(
            self.state.token(),
            SyntaxKind::AsteriskEqualsToken,
            "rescan_asterisk_equals_token requires a `*=` token"
        );
        self.rescan_as(SyntaxKind::EqualsToken, 1)
    }

    /// Splits a private identifier into a `#` token.
    pub fn rescan_hash_token(&mut self) -> SyntaxKind {
        if self.state.token() == SyntaxKind::PrivateIdentifier {
            self.rescan_as(SyntaxKind::HashToken, 1)
        } else {
            self.state.token()
        }
    }

    /// Splits `??` into a single `?`.
    ///
    /// # Panics
    ///
    /// Panics when the current token is not `??`.
    pub fn rescan_question_token(&mut self) -> SyntaxKind {
        assert_eq!(
            self.state.token(),
            SyntaxKind::QuestionQuestionToken,
            "rescan_question_token requires a `??` token"
        );
        self.rescan_as(SyntaxKind::QuestionToken, 1)
    }

    /// Rescans a `/` or `/=` token as a regular expression literal without validating its body.
    pub fn rescan_slash_token(&mut self) -> SyntaxKind {
        if !matches!(
            self.state.token(),
            SyntaxKind::SlashToken | SyntaxKind::SlashEqualsToken
        ) {
            return self.state.token();
        }
        let body_start = self.state.token_start() + 1;
        let body_end = self.find_regular_expression_body_end(body_start);
        let end = if self.state.flags().intersects(TokenFlags::UNTERMINATED) {
            let end = self.recover_unterminated_regular_expression(body_start, body_end);
            let start = self.state.token_start();
            self.error_at(
                diagnostics::UNTERMINATED_REGULAR_EXPRESSION_LITERAL,
                start,
                end - start,
                &[],
            );
            end
        } else {
            self.skip_regular_expression_flags(body_end + 1)
        };
        self.state.set_pos(end);
        let value = self.text[self.state.token_start()..end].to_owned();
        self.state.set_token_value(value);
        self.state
            .finish_token(SyntaxKind::RegularExpressionLiteral);
        SyntaxKind::RegularExpressionLiteral
    }

    fn rescan_as(&mut self, token: SyntaxKind, length: usize) -> SyntaxKind {
        self.state.set_pos(self.state.token_start() + length);
        self.state.finish_token(token);
        token
    }

    /// Finds the closing slash of a regular expression body, flagging it unterminated if absent.
    fn find_regular_expression_body_end(&mut self, body_start: usize) -> usize {
        let bytes = self.text.as_bytes();
        let mut pos = body_start;
        let mut in_escape = false;
        let mut in_character_class = false;
        while let Some(&byte) = bytes.get(pos) {
            if is_line_break(char::from(byte)) {
                break;
            }
            if in_escape {
                in_escape = false;
            } else if byte == b'/' && !in_character_class {
                return pos;
            } else if byte == b'[' {
                in_character_class = true;
            } else if byte == b'\\' {
                in_escape = true;
            } else if byte == b']' {
                in_character_class = false;
            }
            pos += 1;
        }
        self.state.add_flags(TokenFlags::UNTERMINATED);
        pos
    }

    /// Chooses a recovery end for an unterminated regular expression at the first unbalanced
    /// bracket, excluding trailing whitespace and semicolons.
    fn recover_unterminated_regular_expression(&self, body_start: usize, body_end: usize) -> usize {
        let bytes = self.text.as_bytes();
        let mut pos = body_start;
        let mut in_escape = false;
        let mut class_depth = 0_usize;
        let mut in_decimal_quantifier = false;
        let mut group_depth = 0_usize;
        while pos < body_end {
            let byte = bytes[pos];
            if in_escape {
                in_escape = false;
            } else if byte == b'\\' {
                in_escape = true;
            } else if byte == b'[' {
                class_depth += 1;
            } else if byte == b']' && class_depth != 0 {
                class_depth -= 1;
            } else if class_depth == 0 {
                if byte == b'{' {
                    in_decimal_quantifier = true;
                } else if byte == b'}' && in_decimal_quantifier {
                    in_decimal_quantifier = false;
                } else if !in_decimal_quantifier {
                    if byte == b'(' {
                        group_depth += 1;
                    } else if byte == b')' && group_depth != 0 {
                        group_depth -= 1;
                    } else if matches!(byte, b')' | b']' | b'}') {
                        break;
                    }
                }
            }
            pos += 1;
        }
        while pos > body_start {
            let Some(last) = self.text[..pos].chars().next_back() else {
                break;
            };
            if !is_white_space_like(last) && last != ';' {
                break;
            }
            pos -= last.len_utf8();
        }
        pos
    }

    fn skip_regular_expression_flags(&self, mut pos: usize) -> usize {
        while let Some(character) = self.text[pos..].chars().next() {
            if character == char::REPLACEMENT_CHARACTER || !is_identifier_part(character) {
                break;
            }
            pos += character.len_utf8();
        }
        pos
    }
}
