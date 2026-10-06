//! Identifiers, names, literals, and keyword expressions.

use crate::ast::{
    BigIntLiteral, ComputedPropertyName, Identifier, NoSubstitutionTemplateLiteral, NodeData,
    NodeId, NumericLiteral, PrivateIdentifier, RegularExpressionLiteral, StringLiteral, SyntaxKind,
    TokenFlags,
};
use crate::diagnostics::{self, Message};

use super::Parser;

impl Parser<'_> {
    pub(super) fn parse_identifier(&mut self) -> NodeId {
        self.parse_identifier_with_diagnostic(None, None)
    }

    pub(super) fn parse_identifier_with_diagnostic(
        &mut self,
        message: Option<Message>,
        private_identifier_message: Option<Message>,
    ) -> NodeId {
        let is_identifier = self.is_identifier();
        self.create_identifier_with_diagnostic(is_identifier, message, private_identifier_message)
    }

    pub(super) fn parse_identifier_name(&mut self) -> NodeId {
        let is_identifier = self.token.is_identifier_or_keyword();
        self.create_identifier_with_diagnostic(is_identifier, None, None)
    }

    pub(super) fn parse_identifier_name_error_on_unicode_escape_sequence(&mut self) -> NodeId {
        if self
            .scanner
            .token_flags()
            .intersects(TokenFlags::UNICODE_ESCAPE | TokenFlags::EXTENDED_UNICODE_ESCAPE)
        {
            self.parse_error_at_current_token(
                diagnostics::UNICODE_ESCAPE_SEQUENCE_CANNOT_APPEAR_HERE,
                &[],
            );
        }
        let is_identifier = self.token.is_identifier_or_keyword();
        self.create_identifier_with_diagnostic(is_identifier, None, None)
    }

    pub(super) fn parse_binding_identifier(&mut self) -> NodeId {
        self.parse_binding_identifier_with_diagnostic(None)
    }

    pub(super) fn parse_binding_identifier_with_diagnostic(
        &mut self,
        private_identifier_message: Option<Message>,
    ) -> NodeId {
        let saved = self.statement_has_await_identifier;
        let is_identifier = self.is_binding_identifier();
        let id =
            self.create_identifier_with_diagnostic(is_identifier, None, private_identifier_message);
        self.statement_has_await_identifier = saved;
        id
    }

    pub(super) fn parse_optional_binding_identifier(&mut self) -> Option<NodeId> {
        self.is_binding_identifier()
            .then(|| self.parse_binding_identifier())
    }

    pub(super) fn create_identifier(&mut self, is_identifier: bool) -> NodeId {
        self.create_identifier_with_diagnostic(is_identifier, None, None)
    }

    pub(super) fn parse_identifier_name_with_diagnostic(
        &mut self,
        message: Option<Message>,
    ) -> NodeId {
        let is_identifier = self.token.is_identifier_or_keyword();
        self.create_identifier_with_diagnostic(is_identifier, message, None)
    }

    fn create_identifier_with_diagnostic(
        &mut self,
        is_identifier: bool,
        message: Option<Message>,
        private_identifier_message: Option<Message>,
    ) -> NodeId {
        if is_identifier {
            let pos = if self
                .scanner
                .token_flags()
                .intersects(TokenFlags::PRECEDING_JSDOC_LEADING_ASTERISKS)
            {
                self.scanner.token_start()
            } else {
                self.node_pos()
            };
            let text = self.scanner.token_value().to_owned();
            self.next_token_without_check();
            let data = self.new_identifier(text);
            return self.finish_node(SyntaxKind::Identifier, pos, data);
        }
        if self.token == SyntaxKind::PrivateIdentifier {
            self.parse_error_at_current_token(
                private_identifier_message.unwrap_or(
                    diagnostics::PRIVATE_IDENTIFIERS_ARE_NOT_ALLOWED_OUTSIDE_CLASS_BODIES,
                ),
                &[],
            );
            return self.create_identifier_with_diagnostic(true, None, None);
        }
        // Report at the current position only at end of file; elsewhere, report the token.
        let (pos, end) = if self.token == SyntaxKind::EndOfFile {
            let pos = self.scanner.token_full_start();
            (pos, pos)
        } else {
            (self.scanner.token_start(), self.scanner.token_end())
        };
        if let Some(message) = message {
            self.parse_error_at(pos, end, message, &[]);
        } else if self.token.is_reserved_word() {
            let text = self.scanner.token_text();
            self.parse_error_at(
                pos,
                end,
                diagnostics::IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE,
                &[text],
            );
        } else {
            self.parse_error_at(pos, end, diagnostics::IDENTIFIER_EXPECTED, &[]);
        }
        self.create_missing_identifier()
    }

    fn new_identifier(&mut self, text: String) -> NodeData {
        if text == "await" {
            self.statement_has_await_identifier = true;
        }
        NodeData::Identifier(Identifier { text: text.into() })
    }

    pub(super) fn create_missing_identifier(&mut self) -> NodeId {
        let pos = self.node_pos();
        let data = self.new_identifier(String::new());
        self.finish_node(SyntaxKind::Identifier, pos, data)
    }

    pub(super) fn parse_private_identifier(&mut self) -> NodeId {
        let pos = self.node_pos();
        let text = self.scanner.token_value().into();
        self.next_token();
        self.finish_node(
            SyntaxKind::PrivateIdentifier,
            pos,
            NodeData::PrivateIdentifier(PrivateIdentifier { text }),
        )
    }

    pub(super) fn parse_right_side_of_dot(
        &mut self,
        allow_identifier_names: bool,
        allow_private_identifiers: bool,
        allow_unicode_escape_sequence_in_identifier_name: bool,
    ) -> NodeId {
        // `name.` followed by a line break and two identifiers on one line most likely starts a
        // new construct, so the identifier after the dot is missing.
        if self.has_preceding_line_break()
            && self.token.is_identifier_or_keyword()
            && self.look_ahead(Self::next_token_is_identifier_or_keyword_on_same_line)
        {
            let pos = self.node_pos();
            self.parse_error_at(pos, pos, diagnostics::IDENTIFIER_EXPECTED, &[]);
            return self.create_missing_identifier();
        }
        if self.token == SyntaxKind::PrivateIdentifier {
            let node = self.parse_private_identifier();
            if allow_private_identifiers {
                return node;
            }
            let pos = self.node_pos();
            self.parse_error_at(pos, pos, diagnostics::IDENTIFIER_EXPECTED, &[]);
            return self.create_missing_identifier();
        }
        if allow_identifier_names {
            return if allow_unicode_escape_sequence_in_identifier_name {
                self.parse_identifier_name()
            } else {
                self.parse_identifier_name_error_on_unicode_escape_sequence()
            };
        }
        let saved = self.statement_has_await_identifier;
        let id = self.parse_identifier();
        self.statement_has_await_identifier = saved;
        id
    }

    pub(super) fn parse_property_name(&mut self) -> NodeId {
        let saved = self.statement_has_await_identifier;
        let property = self.parse_property_name_worker(true);
        self.statement_has_await_identifier = saved;
        property
    }

    pub(super) fn parse_property_name_worker(
        &mut self,
        allow_computed_property_names: bool,
    ) -> NodeId {
        match self.token {
            SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral | SyntaxKind::BigIntLiteral => {
                self.parse_literal_expression()
            }
            SyntaxKind::OpenBracketToken if allow_computed_property_names => {
                self.parse_computed_property_name()
            }
            SyntaxKind::PrivateIdentifier => self.parse_private_identifier(),
            _ => self.parse_identifier_name(),
        }
    }

    fn parse_computed_property_name(&mut self) -> NodeId {
        let pos = self.node_pos();
        self.parse_expected(SyntaxKind::OpenBracketToken);
        // Any expression is parsed; the grammar checker rejects comma expressions.
        let expression = self.parse_expression_allow_in();
        self.parse_expected(SyntaxKind::CloseBracketToken);
        self.finish_node(
            SyntaxKind::ComputedPropertyName,
            pos,
            NodeData::ComputedPropertyName(ComputedPropertyName { expression }),
        )
    }

    pub(super) fn parse_keyword_expression(&mut self) -> NodeId {
        let pos = self.node_pos();
        let kind = self.token;
        self.next_token();
        self.finish_node(kind, pos, NodeData::KeywordExpression)
    }

    pub(super) fn parse_literal_expression(&mut self) -> NodeId {
        let pos = self.node_pos();
        let text: Box<str> = self.scanner.token_value().into();
        let token_flags = self.scanner.token_flags();
        let kind = self.token;
        let data = match kind {
            SyntaxKind::StringLiteral => {
                NodeData::StringLiteral(StringLiteral { text, token_flags })
            }
            SyntaxKind::NumericLiteral => {
                NodeData::NumericLiteral(NumericLiteral { text, token_flags })
            }
            SyntaxKind::BigIntLiteral => {
                NodeData::BigIntLiteral(BigIntLiteral { text, token_flags })
            }
            SyntaxKind::RegularExpressionLiteral => {
                NodeData::RegularExpressionLiteral(RegularExpressionLiteral { text, token_flags })
            }
            SyntaxKind::NoSubstitutionTemplateLiteral => {
                NodeData::NoSubstitutionTemplateLiteral(NoSubstitutionTemplateLiteral {
                    text,
                    template_flags: token_flags,
                })
            }
            _ => unreachable!("{kind:?} is not a literal token"),
        };
        self.next_token();
        self.finish_node(kind, pos, data)
    }
}
