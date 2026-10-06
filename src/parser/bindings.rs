//! Binding names and destructuring patterns.

use crate::ast::{BindingElement, BindingPattern, NodeData, NodeFlags, NodeId, SyntaxKind};
use crate::diagnostics::Message;

use super::{Parser, ParsingContext};

impl Parser<'_> {
    pub(super) fn parse_identifier_or_pattern(&mut self) -> NodeId {
        self.parse_identifier_or_pattern_with_diagnostic(None)
    }

    pub(super) fn parse_identifier_or_pattern_with_diagnostic(
        &mut self,
        private_identifier_message: Option<Message>,
    ) -> NodeId {
        match self.token {
            SyntaxKind::OpenBracketToken => self.parse_binding_pattern(
                SyntaxKind::ArrayBindingPattern,
                ParsingContext::ArrayBindingElements,
            ),
            SyntaxKind::OpenBraceToken => self.parse_binding_pattern(
                SyntaxKind::ObjectBindingPattern,
                ParsingContext::ObjectBindingElements,
            ),
            _ => self.parse_binding_identifier_with_diagnostic(private_identifier_message),
        }
    }

    fn parse_binding_pattern(&mut self, kind: SyntaxKind, context: ParsingContext) -> NodeId {
        let (open, close) = if kind == SyntaxKind::ArrayBindingPattern {
            (SyntaxKind::OpenBracketToken, SyntaxKind::CloseBracketToken)
        } else {
            (SyntaxKind::OpenBraceToken, SyntaxKind::CloseBraceToken)
        };
        let pos = self.node_pos();
        self.parse_expected(open);
        let elements = self.do_in_context(NodeFlags::DISALLOW_IN_CONTEXT, false, |parser| {
            parser.parse_delimited_list(context, |parser| {
                Some(if kind == SyntaxKind::ArrayBindingPattern {
                    parser.parse_array_binding_element()
                } else {
                    parser.parse_object_binding_element()
                })
            })
        });
        let elements = elements.expect("binding elements always parse");
        self.parse_expected(close);
        self.finish_node(
            kind,
            pos,
            NodeData::BindingPattern(BindingPattern { elements }),
        )
    }

    fn parse_array_binding_element(&mut self) -> NodeId {
        let pos = self.node_pos();
        // An elision (`[, x]`) is an element with no parts.
        let (dot_dot_dot_token, name, initializer) = if self.token == SyntaxKind::CommaToken {
            (None, None, None)
        } else {
            let dot_dot_dot_token = self.parse_optional_token(SyntaxKind::DotDotDotToken);
            let name = self.parse_identifier_or_pattern();
            let initializer = self.parse_initializer();
            (dot_dot_dot_token, Some(name), initializer)
        };
        self.finish_binding_element(pos, dot_dot_dot_token, None, name, initializer)
    }

    fn parse_object_binding_element(&mut self) -> NodeId {
        let pos = self.node_pos();
        let dot_dot_dot_token = self.parse_optional_token(SyntaxKind::DotDotDotToken);
        let token_is_identifier = self.is_binding_identifier();
        let property_name = self.parse_property_name();
        let (property_name, name) = if token_is_identifier && self.token != SyntaxKind::ColonToken {
            (None, property_name)
        } else {
            self.parse_expected(SyntaxKind::ColonToken);
            (Some(property_name), self.parse_identifier_or_pattern())
        };
        let initializer = self.parse_initializer();
        self.finish_binding_element(
            pos,
            dot_dot_dot_token,
            property_name,
            Some(name),
            initializer,
        )
    }

    fn finish_binding_element(
        &mut self,
        pos: usize,
        dot_dot_dot_token: Option<NodeId>,
        property_name: Option<NodeId>,
        name: Option<NodeId>,
        initializer: Option<NodeId>,
    ) -> NodeId {
        self.finish_node(
            SyntaxKind::BindingElement,
            pos,
            NodeData::BindingElement(BindingElement {
                dot_dot_dot_token,
                property_name,
                name,
                initializer,
            }),
        )
    }
}
