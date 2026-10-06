//! Template literal scanning.

use crate::ast::{SyntaxKind, TokenFlags};
use crate::diagnostics;

use super::Scanner;
use super::strings::EscapeOptions;

impl Scanner<'_> {
    /// Scans a template literal part starting at a backtick or closing brace and stores its
    /// cooked value.
    pub(super) fn scan_template_and_set_token_value(
        &mut self,
        should_emit_invalid_escape_error: bool,
    ) -> SyntaxKind {
        let started_with_backtick = self.byte_at(0) == Some(b'`');
        self.state.pos += 1;
        let mut start = self.state.pos;
        let mut value = String::new();
        let token = loop {
            match self.byte_at(0) {
                None | Some(b'`') => {
                    value.push_str(&self.text[start..self.state.pos]);
                    if self.byte_at(0).is_some() {
                        self.state.pos += 1;
                    } else {
                        self.state.token_flags |= TokenFlags::UNTERMINATED;
                        self.error(diagnostics::UNTERMINATED_TEMPLATE_LITERAL);
                    }
                    break if started_with_backtick {
                        SyntaxKind::NoSubstitutionTemplateLiteral
                    } else {
                        SyntaxKind::TemplateTail
                    };
                }
                Some(b'$') if self.byte_at(1) == Some(b'{') => {
                    value.push_str(&self.text[start..self.state.pos]);
                    self.state.pos += 2;
                    break if started_with_backtick {
                        SyntaxKind::TemplateHead
                    } else {
                        SyntaxKind::TemplateMiddle
                    };
                }
                Some(b'\\') => {
                    value.push_str(&self.text[start..self.state.pos]);
                    let options = EscapeOptions {
                        report_errors: should_emit_invalid_escape_error,
                        ..EscapeOptions::STRING
                    };
                    value.push_str(&self.scan_escape_sequence(options));
                    start = self.state.pos;
                }
                Some(b'\r') => {
                    value.push_str(&self.text[start..self.state.pos]);
                    self.state.pos += 1;
                    if self.byte_at(0) == Some(b'\n') {
                        self.state.pos += 1;
                    }
                    value.push('\n');
                    start = self.state.pos;
                }
                Some(_) => self.state.pos += 1,
            }
        };
        self.state.token_value = value;
        token
    }
}
