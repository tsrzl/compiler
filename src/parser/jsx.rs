//! JSX elements, fragments, attributes, and children.

use crate::ast::{
    BinaryExpression, Identifier, JsxAttribute, JsxAttributes, JsxClosingElement, JsxElement,
    JsxExpression, JsxFragment, JsxNamespacedName, JsxOpeningElement, JsxSelfClosingElement,
    JsxSpreadAttribute, JsxText, NodeData, NodeFlags, NodeId, NodeList, PropertyAccessExpression,
    SyntaxKind,
};
use crate::diagnostics;

use super::{Parser, ParsingContext};

impl Parser<'_> {
    /// Parses a JSX element, self-closing element, or fragment. A following element in an
    /// expression context is reported and joined with a comma for recovery.
    pub(super) fn parse_jsx_element_or_self_closing_element_or_fragment(
        &mut self,
        in_expression_context: bool,
        top_invalid_node_position: Option<usize>,
        opening_tag: Option<NodeId>,
        must_be_unary: bool,
    ) -> NodeId {
        let pos = self.node_pos();
        let opening = self
            .parse_jsx_opening_or_self_closing_element_or_opening_fragment(in_expression_context);
        let mut result = match self.builder.node(opening).kind() {
            SyntaxKind::JsxOpeningElement => {
                self.parse_jsx_element_rest(pos, opening, opening_tag, in_expression_context)
            }
            SyntaxKind::JsxOpeningFragment => {
                let children = self.parse_jsx_children(opening);
                let closing_fragment = self.parse_jsx_closing_fragment(in_expression_context);
                self.finish_node(
                    SyntaxKind::JsxFragment,
                    pos,
                    NodeData::JsxFragment(JsxFragment {
                        opening_fragment: opening,
                        children,
                        closing_fragment,
                    }),
                )
            }
            _ => opening,
        };
        // `<div></div><div></div>` in an expression context would otherwise parse the second
        // element as a `<` comparison; wrap both in a comma expression and report it instead.
        if !must_be_unary && in_expression_context && self.token == SyntaxKind::LessThanToken {
            let top_bad_pos = top_invalid_node_position
                .unwrap_or_else(|| self.builder.node(result).pos() as usize);
            let invalid_element = self.parse_jsx_element_or_self_closing_element_or_fragment(
                true,
                Some(top_bad_pos),
                None,
                false,
            );
            let invalid = self.builder.node(invalid_element);
            let (invalid_pos, invalid_end) = (invalid.pos(), invalid.end() as usize);
            let operator_token = self.builder.add_node(
                SyntaxKind::CommaToken,
                invalid_pos,
                invalid_pos,
                NodeFlags::NONE,
                NodeData::Token,
            );
            let start = self.skip_trivia(top_bad_pos);
            self.parse_error_at(
                start,
                invalid_end,
                diagnostics::JSX_EXPRESSIONS_MUST_HAVE_ONE_PARENT_ELEMENT,
                &[],
            );
            result = self.finish_node(
                SyntaxKind::BinaryExpression,
                pos,
                NodeData::BinaryExpression(BinaryExpression {
                    modifiers: None,
                    left: result,
                    type_node: None,
                    operator_token,
                    right: invalid_element,
                }),
            );
        }
        result
    }

    fn parse_jsx_element_rest(
        &mut self,
        pos: usize,
        opening: NodeId,
        opening_tag: Option<NodeId>,
        in_expression_context: bool,
    ) -> NodeId {
        let mut children = self.parse_jsx_children(opening);
        let opening_name = self.jsx_tag_name(opening);
        // An unclosed child that took this element's closing tag is restructured from
        // `<div>(...<span>...</div>)` into `<div>(...<span>...</>)</div>`; the parent reports it.
        let closing_element = if let Some(last_child) =
            self.mismatched_last_child(children, opening_name)
        {
            let (closing_element, new_last) = self.close_child_with_missing_tag(last_child);
            let nodes = self.builder.list(children);
            let mut nodes = nodes[..nodes.len() - 1].to_vec();
            nodes.push(new_last);
            let end = self.builder.node(new_last).end() as usize;
            children = self.new_node_list(children.pos() as usize, end, nodes);
            closing_element
        } else {
            let closing_element = self.parse_jsx_closing_element(opening, in_expression_context);
            let closing_name = self.jsx_tag_name(closing_element);
            if !self.tag_names_are_equivalent(opening_name, closing_name) {
                self.report_mismatched_jsx_closing_tag(opening_name, closing_name, opening_tag);
            }
            closing_element
        };
        self.finish_node(
            SyntaxKind::JsxElement,
            pos,
            NodeData::JsxElement(JsxElement {
                opening_element: opening,
                children,
                closing_element,
            }),
        )
    }

    /// Returns the last child when it is an unclosed element that consumed `opening_name`'s
    /// closing tag.
    fn mismatched_last_child(&self, children: NodeList, opening_name: NodeId) -> Option<NodeId> {
        let &last_child = self.builder.list(children).last()?;
        self.mismatched_last_child_is(last_child, opening_name)
            .then_some(last_child)
    }

    /// Rebuilds `child` with an empty closing tag, returning the closing tag it had consumed and
    /// the rebuilt child.
    fn close_child_with_missing_tag(&mut self, child: NodeId) -> (NodeId, NodeId) {
        let element = self
            .builder
            .node(child)
            .data()
            .as_jsx_element()
            .expect("the mismatched child is a JSX element")
            .clone();
        let end = element.children.end() as usize;
        let missing_identifier = self.finish_node_with_end(
            SyntaxKind::Identifier,
            end,
            end,
            NodeData::Identifier(Identifier { text: "".into() }),
        );
        let new_closing_element = self.finish_node_with_end(
            SyntaxKind::JsxClosingElement,
            end,
            end,
            NodeData::JsxClosingElement(JsxClosingElement {
                tag_name: missing_identifier,
            }),
        );
        let opening_pos = self.builder.node(element.opening_element).pos() as usize;
        let new_last = self.finish_node_with_end(
            SyntaxKind::JsxElement,
            opening_pos,
            end,
            NodeData::JsxElement(JsxElement {
                opening_element: element.opening_element,
                children: element.children,
                closing_element: new_closing_element,
            }),
        );
        (element.closing_element, new_last)
    }

    fn report_mismatched_jsx_closing_tag(
        &mut self,
        opening_name: NodeId,
        closing_name: NodeId,
        opening_tag: Option<NodeId>,
    ) {
        let opening_text = self.node_text_without_trivia(opening_name);
        let opening_matches_parent = opening_tag.is_some_and(|tag| {
            self.builder.node(tag).kind() == SyntaxKind::JsxOpeningElement
                && self.tag_names_are_equivalent(closing_name, self.jsx_tag_name(tag))
        });
        // An opening tag matched with its parent's closing tag gets the error; otherwise the
        // closing tag does.
        let (target, message) = if opening_matches_parent {
            (
                opening_name,
                diagnostics::JSX_ELEMENT_0_HAS_NO_CORRESPONDING_CLOSING_TAG,
            )
        } else {
            (
                closing_name,
                diagnostics::EXPECTED_CORRESPONDING_JSX_CLOSING_TAG_FOR_0,
            )
        };
        let node = self.builder.node(target);
        let (pos, end) = (node.pos() as usize, node.end() as usize);
        self.parse_error_at(pos, end, message, &[&opening_text]);
    }

    fn parse_jsx_children(&mut self, opening_tag: NodeId) -> NodeList {
        let pos = self.node_pos();
        let saved_parsing_contexts = self.parsing_contexts;
        self.parsing_contexts |= ParsingContext::JsxChildren.bit();
        let mut list = Vec::new();
        loop {
            let token = self.rescan_jsx_token(true);
            let Some(child) = self.parse_jsx_child(opening_tag, token) else {
                break;
            };
            list.push(child);
            // Stop after a mismatched child such as `<div>...(<span></div>)` so that the
            // `</div>` can be reattached higher.
            if self.builder.node(opening_tag).kind() == SyntaxKind::JsxOpeningElement {
                let opening_name = self.jsx_tag_name(opening_tag);
                if self.mismatched_last_child_is(child, opening_name) {
                    break;
                }
            }
        }
        self.parsing_contexts = saved_parsing_contexts;
        let end = self.node_pos();
        self.new_node_list(pos, end, list)
    }

    fn mismatched_last_child_is(&self, child: NodeId, opening_name: NodeId) -> bool {
        let Some(element) = self.builder.node(child).data().as_jsx_element() else {
            return false;
        };
        let child_opening = self.jsx_tag_name(element.opening_element);
        let child_closing = self.jsx_tag_name(element.closing_element);
        !self.tag_names_are_equivalent(child_opening, child_closing)
            && self.tag_names_are_equivalent(opening_name, child_closing)
    }

    fn parse_jsx_child(&mut self, opening_tag: NodeId, token: SyntaxKind) -> Option<NodeId> {
        match token {
            SyntaxKind::EndOfFile => {
                self.report_unclosed_jsx_tag(opening_tag);
                None
            }
            SyntaxKind::LessThanSlashToken | SyntaxKind::ConflictMarkerTrivia => None,
            SyntaxKind::JsxText | SyntaxKind::JsxTextAllWhiteSpaces => Some(self.parse_jsx_text()),
            SyntaxKind::OpenBraceToken => self.parse_jsx_expression(false),
            SyntaxKind::LessThanToken => {
                Some(self.parse_jsx_element_or_self_closing_element_or_fragment(
                    false,
                    None,
                    Some(opening_tag),
                    false,
                ))
            }
            _ => unreachable!("{token:?} is not a JSX child token"),
        }
    }

    /// Reports end of file at the tag that lacks a closing element.
    fn report_unclosed_jsx_tag(&mut self, opening_tag: NodeId) {
        let opening = self.builder.node(opening_tag);
        if opening.kind() == SyntaxKind::JsxOpeningFragment {
            let (pos, end) = (opening.pos() as usize, opening.end() as usize);
            self.parse_error_at(
                pos,
                end,
                diagnostics::JSX_FRAGMENT_HAS_NO_CORRESPONDING_CLOSING_TAG,
                &[],
            );
            return;
        }
        // Report only the tag name, such as `Foo.Bar` in `< Foo.Bar >`.
        let tag = self.jsx_tag_name(opening_tag);
        let tag_node = self.builder.node(tag);
        let end = tag_node.end() as usize;
        let start = self.skip_trivia(tag_node.pos() as usize).min(end);
        let text = self.node_text_without_trivia(tag);
        self.parse_error_at(
            start,
            end,
            diagnostics::JSX_ELEMENT_0_HAS_NO_CORRESPONDING_CLOSING_TAG,
            &[&text],
        );
    }

    fn parse_jsx_text(&mut self) -> NodeId {
        let pos = self.node_pos();
        let data = NodeData::JsxText(JsxText {
            text: self.scanner.token_value().into(),
            contains_only_trivia_white_spaces: self.token == SyntaxKind::JsxTextAllWhiteSpaces,
        });
        self.scan_jsx_text();
        self.finish_node(SyntaxKind::JsxText, pos, data)
    }

    fn parse_jsx_expression(&mut self, in_expression_context: bool) -> Option<NodeId> {
        let pos = self.node_pos();
        if !self.parse_expected(SyntaxKind::OpenBraceToken) {
            return None;
        }
        let mut dot_dot_dot_token = None;
        let mut expression = None;
        if self.token != SyntaxKind::CloseBraceToken {
            if !in_expression_context {
                dot_dot_dot_token = self.parse_optional_token(SyntaxKind::DotDotDotToken);
            }
            // Only an assignment expression is valid, but a comma sequence is parsed so the
            // grammar checker can give a better error.
            expression = Some(self.parse_expression());
        }
        if in_expression_context {
            self.parse_expected(SyntaxKind::CloseBraceToken);
        } else if self.parse_expected_with_diagnostic(SyntaxKind::CloseBraceToken, None, false) {
            self.scan_jsx_text();
        }
        Some(self.finish_node(
            SyntaxKind::JsxExpression,
            pos,
            NodeData::JsxExpression(JsxExpression {
                dot_dot_dot_token,
                expression,
            }),
        ))
    }

    fn parse_jsx_closing_element(
        &mut self,
        opening: NodeId,
        in_expression_context: bool,
    ) -> NodeId {
        let pos = self.node_pos();
        self.parse_expected(SyntaxKind::LessThanSlashToken);
        let tag_name = self.parse_jsx_element_name();
        if self.parse_expected_with_diagnostic(SyntaxKind::GreaterThanToken, None, false) {
            // Advance manually so that text after a matching closing tag is scanned as JSX.
            let opening_name = self.jsx_tag_name(opening);
            if in_expression_context || !self.tag_names_are_equivalent(opening_name, tag_name) {
                self.next_token();
            } else {
                self.scan_jsx_text();
            }
        }
        self.finish_node(
            SyntaxKind::JsxClosingElement,
            pos,
            NodeData::JsxClosingElement(JsxClosingElement { tag_name }),
        )
    }

    fn parse_jsx_opening_or_self_closing_element_or_opening_fragment(
        &mut self,
        in_expression_context: bool,
    ) -> NodeId {
        let pos = self.node_pos();
        self.parse_expected(SyntaxKind::LessThanToken);
        if self.token == SyntaxKind::GreaterThanToken {
            self.scan_jsx_text();
            return self.finish_node(
                SyntaxKind::JsxOpeningFragment,
                pos,
                NodeData::JsxOpeningFragment,
            );
        }
        let tag_name = self.parse_jsx_element_name();
        let type_arguments = if self.in_context(NodeFlags::JAVA_SCRIPT_FILE) {
            None
        } else {
            self.parse_type_arguments()
        };
        let attributes = self.parse_jsx_attributes();
        if self.token == SyntaxKind::GreaterThanToken {
            // Scan the text after `>` as JSX so that characters such as `#` are not reported as
            // scanning errors.
            self.scan_jsx_text();
            return self.finish_node(
                SyntaxKind::JsxOpeningElement,
                pos,
                NodeData::JsxOpeningElement(JsxOpeningElement {
                    tag_name,
                    type_arguments,
                    attributes,
                }),
            );
        }
        self.parse_expected(SyntaxKind::SlashToken);
        if self.parse_expected_with_diagnostic(SyntaxKind::GreaterThanToken, None, false) {
            if in_expression_context {
                self.next_token();
            } else {
                self.scan_jsx_text();
            }
        }
        self.finish_node(
            SyntaxKind::JsxSelfClosingElement,
            pos,
            NodeData::JsxSelfClosingElement(JsxSelfClosingElement {
                tag_name,
                type_arguments,
                attributes,
            }),
        )
    }

    /// Parses a tag name: an identifier, `this`, a namespaced name, or a dotted access.
    fn parse_jsx_element_name(&mut self) -> NodeId {
        let pos = self.node_pos();
        let initial = self.parse_jsx_tag_name();
        // `a:b.c` is invalid; attribute parsing reports the unexpected `:`.
        if self.builder.node(initial).kind() == SyntaxKind::JsxNamespacedName {
            return initial;
        }
        let mut expression = initial;
        while self.parse_optional(SyntaxKind::DotToken) {
            let name = self.parse_right_side_of_dot(true, false, false);
            expression = self.finish_node(
                SyntaxKind::PropertyAccessExpression,
                pos,
                NodeData::PropertyAccessExpression(PropertyAccessExpression {
                    expression,
                    question_dot_token: None,
                    name,
                }),
            );
        }
        expression
    }

    fn parse_jsx_tag_name(&mut self) -> NodeId {
        let pos = self.node_pos();
        self.scan_jsx_identifier();
        let is_this = self.token == SyntaxKind::ThisKeyword;
        let tag_name = self.parse_identifier_name_error_on_unicode_escape_sequence();
        if self.parse_optional(SyntaxKind::ColonToken) {
            self.scan_jsx_identifier();
            let name = self.parse_identifier_name_error_on_unicode_escape_sequence();
            return self.finish_node(
                SyntaxKind::JsxNamespacedName,
                pos,
                NodeData::JsxNamespacedName(JsxNamespacedName {
                    namespace: tag_name,
                    name,
                }),
            );
        }
        if is_this {
            return self.finish_node(SyntaxKind::ThisKeyword, pos, NodeData::KeywordExpression);
        }
        tag_name
    }

    fn parse_jsx_attributes(&mut self) -> NodeId {
        let pos = self.node_pos();
        let properties = self.parse_list(ParsingContext::JsxAttributes, Self::parse_jsx_attribute);
        self.finish_node(
            SyntaxKind::JsxAttributes,
            pos,
            NodeData::JsxAttributes(JsxAttributes { properties }),
        )
    }

    fn parse_jsx_attribute(&mut self) -> NodeId {
        let pos = self.node_pos();
        if self.token == SyntaxKind::OpenBraceToken {
            self.parse_expected(SyntaxKind::OpenBraceToken);
            self.parse_expected(SyntaxKind::DotDotDotToken);
            let expression = self.parse_expression();
            self.parse_expected(SyntaxKind::CloseBraceToken);
            return self.finish_node(
                SyntaxKind::JsxSpreadAttribute,
                pos,
                NodeData::JsxSpreadAttribute(JsxSpreadAttribute { expression }),
            );
        }
        let name = self.parse_jsx_attribute_name();
        let initializer = self.parse_jsx_attribute_value();
        self.finish_node(
            SyntaxKind::JsxAttribute,
            pos,
            NodeData::JsxAttribute(JsxAttribute { name, initializer }),
        )
    }

    fn parse_jsx_attribute_name(&mut self) -> NodeId {
        let pos = self.node_pos();
        self.scan_jsx_identifier();
        let name = self.parse_identifier_name_error_on_unicode_escape_sequence();
        if !self.parse_optional(SyntaxKind::ColonToken) {
            return name;
        }
        self.scan_jsx_identifier();
        let local_name = self.parse_identifier_name_error_on_unicode_escape_sequence();
        self.finish_node(
            SyntaxKind::JsxNamespacedName,
            pos,
            NodeData::JsxNamespacedName(JsxNamespacedName {
                namespace: name,
                name: local_name,
            }),
        )
    }

    fn parse_jsx_attribute_value(&mut self) -> Option<NodeId> {
        if self.token != SyntaxKind::EqualsToken {
            return None;
        }
        match self.scan_jsx_attribute_value() {
            SyntaxKind::StringLiteral => Some(self.parse_literal_expression()),
            SyntaxKind::OpenBraceToken => self.parse_jsx_expression(true),
            SyntaxKind::LessThanToken => Some(
                self.parse_jsx_element_or_self_closing_element_or_fragment(true, None, None, false),
            ),
            _ => {
                self.parse_error_at_current_token(diagnostics::X_OR_JSX_ELEMENT_EXPECTED, &[]);
                None
            }
        }
    }

    fn parse_jsx_closing_fragment(&mut self, in_expression_context: bool) -> NodeId {
        let pos = self.node_pos();
        self.parse_expected(SyntaxKind::LessThanSlashToken);
        if self.parse_expected_with_diagnostic(
            SyntaxKind::GreaterThanToken,
            Some(diagnostics::EXPECTED_CORRESPONDING_CLOSING_TAG_FOR_JSX_FRAGMENT),
            false,
        ) {
            if in_expression_context {
                self.next_token();
            } else {
                self.scan_jsx_text();
            }
        }
        self.finish_node(
            SyntaxKind::JsxClosingFragment,
            pos,
            NodeData::JsxClosingFragment,
        )
    }

    /// Returns the tag name of an opening, self-closing, or closing element.
    fn jsx_tag_name(&self, element: NodeId) -> NodeId {
        match self.builder.node(element).data() {
            NodeData::JsxOpeningElement(opening) => opening.tag_name,
            NodeData::JsxSelfClosingElement(element) => element.tag_name,
            NodeData::JsxClosingElement(closing) => closing.tag_name,
            data => unreachable!("{data:?} has no JSX tag name"),
        }
    }

    /// Mirrors TypeScript-Go's `ast.TagNamesAreEquivalent`.
    fn tag_names_are_equivalent(&self, left: NodeId, right: NodeId) -> bool {
        let (left_node, right_node) = (self.builder.node(left), self.builder.node(right));
        if left_node.kind() != right_node.kind() {
            return false;
        }
        match (left_node.data(), right_node.data()) {
            (NodeData::Identifier(left), NodeData::Identifier(right)) => left.text == right.text,
            (NodeData::JsxNamespacedName(left), NodeData::JsxNamespacedName(right)) => {
                self.identifier_text(left.namespace) == self.identifier_text(right.namespace)
                    && self.identifier_text(left.name) == self.identifier_text(right.name)
            }
            (
                NodeData::PropertyAccessExpression(left),
                NodeData::PropertyAccessExpression(right),
            ) => {
                self.identifier_text(left.name) == self.identifier_text(right.name)
                    && self.tag_names_are_equivalent(left.expression, right.expression)
            }
            _ => left_node.kind() == SyntaxKind::ThisKeyword,
        }
    }

    /// Returns a node's source text without leading trivia.
    fn node_text_without_trivia(&self, node: NodeId) -> String {
        let node = self.builder.node(node);
        let start = self.skip_trivia(node.pos() as usize);
        self.scanner.text()[start..node.end() as usize].to_owned()
    }

    fn scan_jsx_text(&mut self) -> SyntaxKind {
        self.token = self.scanner.scan_jsx_token();
        self.drain_scanner_diagnostics();
        self.token
    }

    fn rescan_jsx_token(&mut self, allow_multiline_text: bool) -> SyntaxKind {
        self.token = self.scanner.rescan_jsx_token(allow_multiline_text);
        self.drain_scanner_diagnostics();
        self.token
    }

    fn scan_jsx_identifier(&mut self) -> SyntaxKind {
        self.token = self.scanner.scan_jsx_identifier();
        self.drain_scanner_diagnostics();
        self.token
    }

    fn scan_jsx_attribute_value(&mut self) -> SyntaxKind {
        self.token = self.scanner.scan_jsx_attribute_value();
        self.drain_scanner_diagnostics();
        self.token
    }
}
