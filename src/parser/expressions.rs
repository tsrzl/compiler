//! Expression parsing, from assignment expressions down to primary expressions.

use crate::ast::{
    ArrayLiteralExpression, AwaitExpression, BinaryExpression, CallExpression,
    ConditionalExpression, DeleteExpression, ElementAccessExpression, MetaProperty, NewExpression,
    NodeData, NodeFlags, NodeId, NodeList, NonNullExpression, ObjectLiteralExpression,
    OperatorPrecedence, ParenthesizedExpression, PostfixUnaryExpression, PrefixUnaryExpression,
    PropertyAccessExpression, PropertyAssignment, ShorthandPropertyAssignment, SpreadAssignment,
    SpreadElement, SyntaxKind, TaggedTemplateExpression, TemplateExpression, TemplateHead,
    TemplateMiddle, TemplateSpan, TemplateTail, TokenFlags, TypeOfExpression, VoidExpression,
    YieldExpression,
};
use crate::ast::{
    AsExpression, ExpressionWithTypeArguments, FunctionExpression, ModifierFlags,
    SatisfiesExpression, TypeAssertion,
};
use crate::diagnostics;
use crate::scanner::LanguageVariant;

use super::modifiers::ModifierOptions;
use super::signatures::SignatureFlags;

use super::{Parser, ParsingContext, token_to_string};

impl Parser<'_> {
    pub(super) fn parse_expression(&mut self) -> NodeId {
        // A decorator context never extends into a full expression.
        let saved_context_flags = self.context_flags;
        self.context_flags = self.context_flags.without(NodeFlags::DECORATOR_CONTEXT);
        let pos = self.node_pos();
        let mut expression = self.parse_assignment_expression_or_higher();
        while let Some(operator_token) = self.parse_optional_token(SyntaxKind::CommaToken) {
            let right = self.parse_assignment_expression_or_higher();
            expression = self.make_binary_expression(expression, operator_token, right, pos);
        }
        self.context_flags = saved_context_flags;
        expression
    }

    pub(super) fn parse_expression_allow_in(&mut self) -> NodeId {
        self.do_in_context(
            NodeFlags::DISALLOW_IN_CONTEXT,
            false,
            Self::parse_expression,
        )
    }

    pub(super) fn parse_assignment_expression_or_higher(&mut self) -> NodeId {
        self.parse_assignment_expression_or_higher_worker(true)
    }

    pub(super) fn parse_assignment_expression_or_higher_worker(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> NodeId {
        if self.is_yield_expression() {
            return self.parse_yield_expression();
        }
        if let Some(arrow) = self
            .try_parse_parenthesized_arrow_function_expression(allow_return_type_in_arrow_function)
        {
            return arrow;
        }
        if let Some(arrow) = self
            .try_parse_async_simple_arrow_function_expression(allow_return_type_in_arrow_function)
        {
            return arrow;
        }
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        let expression = self.parse_binary_expression_or_higher(OperatorPrecedence::LOWEST);
        // A single unparenthesized arrow parameter (`x => ...`) is recognized only after it has
        // been parsed as an identifier.
        if self.builder.node(expression).kind() == SyntaxKind::Identifier
            && self.token == SyntaxKind::EqualsGreaterThanToken
        {
            return self.parse_simple_arrow_function_expression(
                pos,
                expression,
                allow_return_type_in_arrow_function,
                jsdoc,
                None,
            );
        }
        if self.is_left_hand_side_expression(expression)
            && self.rescan_greater_than_token().is_assignment_operator()
        {
            let operator_token = self.parse_token_node();
            let right = self
                .parse_assignment_expression_or_higher_worker(allow_return_type_in_arrow_function);
            return self.make_binary_expression(expression, operator_token, right, pos);
        }
        self.parse_conditional_expression_rest(expression, pos, allow_return_type_in_arrow_function)
    }

    /// Reports and skips an expression whose syntax has not been ported from TypeScript-Go yet,
    /// producing a missing identifier. It is removed when expression parsing is complete.
    pub(super) fn parse_unported_expression(&mut self, pos: usize) -> NodeId {
        self.parse_error_at_current_token(diagnostics::EXPRESSION_EXPECTED, &[]);
        self.next_token();
        let _ = pos;
        self.create_missing_identifier()
    }

    fn is_yield_expression(&mut self) -> bool {
        if self.token != SyntaxKind::YieldKeyword {
            return false;
        }
        if self.in_context(NodeFlags::YIELD_CONTEXT) {
            return true;
        }
        // Outside a generator, `yield` followed by an operand on the same line can only be a yield
        // expression; it is parsed as one and reported later.
        self.look_ahead(Self::next_token_is_identifier_or_keyword_or_literal_on_same_line)
    }

    fn parse_yield_expression(&mut self) -> NodeId {
        let pos = self.node_pos();
        self.next_token();
        let data = if !self.has_preceding_line_break()
            && (self.token == SyntaxKind::AsteriskToken || self.is_start_of_expression())
        {
            let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);
            let expression = Some(self.parse_assignment_expression_or_higher());
            YieldExpression {
                asterisk_token,
                expression,
            }
        } else {
            YieldExpression {
                asterisk_token: None,
                expression: None,
            }
        };
        self.finish_node(
            SyntaxKind::YieldExpression,
            pos,
            NodeData::YieldExpression(data),
        )
    }

    fn parse_conditional_expression_rest(
        &mut self,
        condition: NodeId,
        pos: usize,
        allow_return_type_in_arrow_function: bool,
    ) -> NodeId {
        let Some(question_token) = self.parse_optional_token(SyntaxKind::QuestionToken) else {
            return condition;
        };
        // `in` is allowed in the true branch but not in the false branch.
        let saved_context_flags = self.context_flags;
        self.set_context_flags(NodeFlags::DISALLOW_IN_CONTEXT, false);
        let when_true = self.parse_assignment_expression_or_higher_worker(false);
        self.context_flags = saved_context_flags;
        let colon_token = self.parse_expected_token(SyntaxKind::ColonToken);
        let when_false = if self.node_is_present(colon_token) {
            self.parse_assignment_expression_or_higher_worker(allow_return_type_in_arrow_function)
        } else {
            self.create_missing_identifier()
        };
        self.finish_node(
            SyntaxKind::ConditionalExpression,
            pos,
            NodeData::ConditionalExpression(ConditionalExpression {
                condition,
                question_token,
                when_true,
                colon_token,
                when_false,
            }),
        )
    }

    pub(super) fn parse_binary_expression_or_higher(
        &mut self,
        precedence: OperatorPrecedence,
    ) -> NodeId {
        let pos = self.node_pos();
        let left = self.parse_unary_expression_or_higher();
        self.parse_binary_expression_rest(precedence, left, pos)
    }

    fn parse_binary_expression_rest(
        &mut self,
        precedence: OperatorPrecedence,
        mut left: NodeId,
        pos: usize,
    ) -> NodeId {
        let mut last_operand = left;
        loop {
            // Rescanning merges `>` with following characters into `>=`, `>>`, and so on.
            let operator = self.rescan_greater_than_token();
            let new_precedence = operator.binary_operator_precedence();
            // `**` is right associative; every other binary operator is left associative.
            let consume = if operator == SyntaxKind::AsteriskAsteriskToken {
                new_precedence >= precedence
            } else {
                new_precedence > precedence
            };
            if !consume
                || (operator == SyntaxKind::InKeyword
                    && self.in_context(NodeFlags::DISALLOW_IN_CONTEXT))
            {
                break;
            }
            if matches!(
                operator,
                SyntaxKind::AsKeyword | SyntaxKind::SatisfiesKeyword
            ) {
                // `as` and `satisfies` after a line break start a new statement through ASI.
                if self.has_preceding_line_break() {
                    break;
                }
                self.next_token();
                // In `a ## b as T`, stop at a following operator that binds tighter than `##`, so
                // that erasing `as T` cannot change the expression's meaning.
                let last_precedence = self
                    .builder
                    .node(last_operand)
                    .data()
                    .as_binary_expression()
                    .map_or(OperatorPrecedence::HIGHEST, |binary| {
                        self.builder
                            .node(binary.operator_token)
                            .kind()
                            .binary_operator_precedence()
                    });
                let type_node = self.parse_type();
                left = self.make_type_assertion_expression(operator, left, type_node);
                if self
                    .rescan_greater_than_token()
                    .binary_operator_precedence()
                    > last_precedence
                {
                    break;
                }
                continue;
            }
            let operator_token = self.parse_token_node();
            let right = self.parse_binary_expression_or_higher(new_precedence);
            left = self.make_binary_expression(left, operator_token, right, pos);
            last_operand = left;
        }
        left
    }

    /// Makes an `as` or `satisfies` expression spanning from its operand.
    fn make_type_assertion_expression(
        &mut self,
        operator: SyntaxKind,
        expression: NodeId,
        type_node: NodeId,
    ) -> NodeId {
        let pos = self.builder.node(expression).pos() as usize;
        let (kind, data) = if operator == SyntaxKind::SatisfiesKeyword {
            (
                SyntaxKind::SatisfiesExpression,
                NodeData::SatisfiesExpression(SatisfiesExpression {
                    expression,
                    type_node,
                }),
            )
        } else {
            (
                SyntaxKind::AsExpression,
                NodeData::AsExpression(AsExpression {
                    expression,
                    type_node,
                }),
            )
        };
        self.finish_node(kind, pos, data)
    }

    fn make_binary_expression(
        &mut self,
        left: NodeId,
        operator_token: NodeId,
        right: NodeId,
        pos: usize,
    ) -> NodeId {
        self.finish_node(
            SyntaxKind::BinaryExpression,
            pos,
            NodeData::BinaryExpression(BinaryExpression {
                modifiers: None,
                left,
                type_node: None,
                operator_token,
                right,
            }),
        )
    }

    pub(super) fn parse_unary_expression_or_higher(&mut self) -> NodeId {
        if self.is_update_expression() {
            let pos = self.node_pos();
            let update_expression = self.parse_update_expression();
            if self.token == SyntaxKind::AsteriskAsteriskToken {
                let precedence = self.token.binary_operator_precedence();
                return self.parse_binary_expression_rest(precedence, update_expression, pos);
            }
            return update_expression;
        }
        let unary_operator = self.token;
        let simple_unary_expression = self.parse_simple_unary_expression();
        if self.token == SyntaxKind::AsteriskAsteriskToken {
            let node = self.builder.node(simple_unary_expression);
            let (kind, end) = (node.kind(), node.end() as usize);
            let pos = self.skip_trivia(node.pos() as usize);
            if kind == SyntaxKind::TypeAssertionExpression {
                self.parse_error_at(
                    pos,
                    end,
                    diagnostics::A_TYPE_ASSERTION_EXPRESSION_IS_NOT_ALLOWED_IN_THE_LEFT_HAND_SIDE_OF_AN_EXPONENTIATION_EXPRESSION_CONSIDER_ENCLOSING_THE_EXPRESSION_IN_PARENTHESES,
                    &[],
                );
            } else {
                self.parse_error_at(
                    pos,
                    end,
                    diagnostics::AN_UNARY_EXPRESSION_WITH_THE_0_OPERATOR_IS_NOT_ALLOWED_IN_THE_LEFT_HAND_SIDE_OF_AN_EXPONENTIATION_EXPRESSION_CONSIDER_ENCLOSING_THE_EXPRESSION_IN_PARENTHESES,
                    &[token_to_string(unary_operator)],
                );
            }
        }
        simple_unary_expression
    }

    fn is_update_expression(&self) -> bool {
        match self.token {
            SyntaxKind::PlusToken
            | SyntaxKind::MinusToken
            | SyntaxKind::TildeToken
            | SyntaxKind::ExclamationToken
            | SyntaxKind::DeleteKeyword
            | SyntaxKind::TypeOfKeyword
            | SyntaxKind::VoidKeyword
            | SyntaxKind::AwaitKeyword => false,
            SyntaxKind::LessThanToken => self.language_variant == LanguageVariant::Jsx,
            _ => true,
        }
    }

    fn parse_update_expression(&mut self) -> NodeId {
        let pos = self.node_pos();
        if matches!(
            self.token,
            SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
        ) {
            let operator = self.token;
            self.next_token();
            let operand = self.parse_left_hand_side_expression_or_higher();
            return self.finish_node(
                SyntaxKind::PrefixUnaryExpression,
                pos,
                NodeData::PrefixUnaryExpression(PrefixUnaryExpression { operator, operand }),
            );
        }
        if self.language_variant == LanguageVariant::Jsx && self.token == SyntaxKind::LessThanToken
        {
            return self.parse_unported_expression(pos);
        }
        let operand = self.parse_left_hand_side_expression_or_higher();
        if matches!(
            self.token,
            SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
        ) && !self.has_preceding_line_break()
        {
            let operator = self.token;
            self.next_token();
            return self.finish_node(
                SyntaxKind::PostfixUnaryExpression,
                pos,
                NodeData::PostfixUnaryExpression(PostfixUnaryExpression { operand, operator }),
            );
        }
        operand
    }

    fn parse_simple_unary_expression(&mut self) -> NodeId {
        let pos = self.node_pos();
        let token = self.token;
        match token {
            SyntaxKind::PlusToken
            | SyntaxKind::MinusToken
            | SyntaxKind::TildeToken
            | SyntaxKind::ExclamationToken => {
                let operator = self.token;
                self.next_token();
                let operand = self.parse_simple_unary_expression();
                self.finish_node(
                    SyntaxKind::PrefixUnaryExpression,
                    pos,
                    NodeData::PrefixUnaryExpression(PrefixUnaryExpression { operator, operand }),
                )
            }
            SyntaxKind::DeleteKeyword => {
                self.next_token();
                let expression = self.parse_simple_unary_expression();
                self.finish_node(
                    SyntaxKind::DeleteExpression,
                    pos,
                    NodeData::DeleteExpression(DeleteExpression { expression }),
                )
            }
            SyntaxKind::TypeOfKeyword => {
                self.next_token();
                let expression = self.parse_simple_unary_expression();
                self.finish_node(
                    SyntaxKind::TypeOfExpression,
                    pos,
                    NodeData::TypeOfExpression(TypeOfExpression { expression }),
                )
            }
            SyntaxKind::VoidKeyword => {
                self.next_token();
                let expression = self.parse_simple_unary_expression();
                self.finish_node(
                    SyntaxKind::VoidExpression,
                    pos,
                    NodeData::VoidExpression(VoidExpression { expression }),
                )
            }
            // JSX elements are not ported yet; elsewhere `<T>expr` is a type assertion.
            SyntaxKind::LessThanToken if self.language_variant == LanguageVariant::Jsx => {
                self.parse_unported_expression(pos)
            }
            SyntaxKind::LessThanToken => self.parse_type_assertion(),
            SyntaxKind::AwaitKeyword if self.is_await_expression() => {
                self.next_token();
                let expression = self.parse_simple_unary_expression();
                self.finish_node(
                    SyntaxKind::AwaitExpression,
                    pos,
                    NodeData::AwaitExpression(AwaitExpression { expression }),
                )
            }
            _ => self.parse_update_expression(),
        }
    }

    fn parse_type_assertion(&mut self) -> NodeId {
        let pos = self.node_pos();
        self.parse_expected(SyntaxKind::LessThanToken);
        let type_node = self.parse_type();
        self.parse_expected(SyntaxKind::GreaterThanToken);
        let expression = self.parse_simple_unary_expression();
        self.finish_node(
            SyntaxKind::TypeAssertionExpression,
            pos,
            NodeData::TypeAssertion(TypeAssertion {
                type_node,
                expression,
            }),
        )
    }

    fn is_await_expression(&mut self) -> bool {
        if self.token != SyntaxKind::AwaitKeyword {
            return false;
        }
        self.in_context(NodeFlags::AWAIT_CONTEXT)
            || self.look_ahead(Self::next_token_is_identifier_or_keyword_or_literal_on_same_line)
    }

    pub(super) fn parse_left_hand_side_expression_or_higher(&mut self) -> NodeId {
        let pos = self.node_pos();
        let expression = if self.token == SyntaxKind::ImportKeyword {
            if self.look_ahead(|parser| {
                matches!(
                    parser.next_token(),
                    SyntaxKind::OpenParenToken | SyntaxKind::LessThanToken
                )
            }) {
                // `import(...)` is a call only when `(` follows, so a statement such as
                // `import * as x from "m"` is not consumed as an expression.
                self.source_flags |= NodeFlags::POSSIBLY_CONTAINS_DYNAMIC_IMPORT;
                self.parse_keyword_expression()
            } else if self.look_ahead(|parser| parser.next_token() == SyntaxKind::DotToken) {
                self.next_token();
                self.next_token();
                let name = self.parse_identifier_name();
                let meta = self.finish_node(
                    SyntaxKind::MetaProperty,
                    pos,
                    NodeData::MetaProperty(MetaProperty {
                        keyword_token: SyntaxKind::ImportKeyword,
                        name,
                    }),
                );
                if self.identifier_text(name) == "defer" {
                    if matches!(
                        self.token,
                        SyntaxKind::OpenParenToken | SyntaxKind::LessThanToken
                    ) {
                        self.source_flags |= NodeFlags::POSSIBLY_CONTAINS_DYNAMIC_IMPORT;
                    }
                } else {
                    self.source_flags |= NodeFlags::POSSIBLY_CONTAINS_IMPORT_META;
                }
                meta
            } else {
                self.parse_member_expression_or_higher()
            }
        } else if self.token == SyntaxKind::SuperKeyword {
            self.parse_super_expression()
        } else {
            self.parse_member_expression_or_higher()
        };
        self.parse_call_expression_rest(pos, expression)
    }

    fn parse_super_expression(&mut self) -> NodeId {
        let pos = self.node_pos();
        let mut expression = self.parse_keyword_expression();
        if self.token == SyntaxKind::LessThanToken {
            let start_pos = self.node_pos();
            if let Some(type_arguments) = self.try_parse_type_arguments_in_expression() {
                let end = self.node_pos();
                self.parse_error_at(
                    start_pos,
                    end,
                    diagnostics::X_SUPER_MAY_NOT_USE_TYPE_ARGUMENTS,
                    &[],
                );
                if !self.is_template_start_of_tagged_template() {
                    expression =
                        self.finish_expression_with_type_arguments(pos, expression, type_arguments);
                }
            }
        }
        if matches!(
            self.token,
            SyntaxKind::OpenParenToken | SyntaxKind::DotToken | SyntaxKind::OpenBracketToken
        ) {
            return expression;
        }
        // `super` must be followed by arguments or member access; recover with a property access.
        self.parse_error_at_current_token(
            diagnostics::X_SUPER_MUST_BE_FOLLOWED_BY_AN_ARGUMENT_LIST_OR_MEMBER_ACCESS,
            &[],
        );
        let name = self.parse_right_side_of_dot(true, true, true);
        self.finish_node(
            SyntaxKind::PropertyAccessExpression,
            pos,
            NodeData::PropertyAccessExpression(PropertyAccessExpression {
                expression,
                question_dot_token: None,
                name,
            }),
        )
    }

    fn is_template_start_of_tagged_template(&self) -> bool {
        matches!(
            self.token,
            SyntaxKind::NoSubstitutionTemplateLiteral | SyntaxKind::TemplateHead
        )
    }

    fn parse_member_expression_or_higher(&mut self) -> NodeId {
        let pos = self.node_pos();
        let expression = self.parse_primary_expression();
        self.parse_member_expression_rest(pos, expression, true)
    }

    pub(super) fn parse_member_expression_rest(
        &mut self,
        pos: usize,
        mut expression: NodeId,
        allow_optional_chain: bool,
    ) -> NodeId {
        loop {
            let mut question_dot_token = None;
            let is_property_access = if allow_optional_chain
                && self.is_start_of_optional_property_or_element_access_chain()
            {
                question_dot_token = Some(self.parse_expected_token(SyntaxKind::QuestionDotToken));
                self.token.is_identifier_or_keyword()
            } else {
                self.parse_optional(SyntaxKind::DotToken)
            };
            if is_property_access {
                expression =
                    self.parse_property_access_expression_rest(pos, expression, question_dot_token);
                continue;
            }
            // In a decorator, `[` may start a computed property name rather than element access.
            if (question_dot_token.is_some() || !self.in_context(NodeFlags::DECORATOR_CONTEXT))
                && self.parse_optional(SyntaxKind::OpenBracketToken)
            {
                expression =
                    self.parse_element_access_expression_rest(pos, expression, question_dot_token);
                continue;
            }
            if self.is_template_start_of_tagged_template() {
                // Type arguments of an instantiation expression move onto the tagged template.
                let (tag, type_arguments) = match self.expression_with_type_arguments(expression) {
                    Some((inner, type_arguments)) if question_dot_token.is_none() => {
                        (inner, type_arguments)
                    }
                    _ => (expression, None),
                };
                expression =
                    self.parse_tagged_template_rest(pos, tag, question_dot_token, type_arguments);
                continue;
            }
            if question_dot_token.is_none() {
                if self.token == SyntaxKind::ExclamationToken && !self.has_preceding_line_break() {
                    self.next_token();
                    expression = self.finish_node(
                        SyntaxKind::NonNullExpression,
                        pos,
                        NodeData::NonNullExpression(NonNullExpression { expression }),
                    );
                    continue;
                }
                if let Some(type_arguments) = self.try_parse_type_arguments_in_expression() {
                    expression =
                        self.finish_expression_with_type_arguments(pos, expression, type_arguments);
                    continue;
                }
            }
            return expression;
        }
    }

    fn is_start_of_optional_property_or_element_access_chain(&mut self) -> bool {
        self.token == SyntaxKind::QuestionDotToken
            && self.look_ahead(|parser| {
                parser.next_token();
                parser.token.is_identifier_or_keyword()
                    || parser.token == SyntaxKind::OpenBracketToken
                    || parser.is_template_start_of_tagged_template()
            })
    }

    fn parse_property_access_expression_rest(
        &mut self,
        pos: usize,
        expression: NodeId,
        question_dot_token: Option<NodeId>,
    ) -> NodeId {
        let name = self.parse_right_side_of_dot(true, true, true);
        let is_optional_chain =
            question_dot_token.is_some() || self.try_reparse_optional_chain(expression);
        if is_optional_chain && self.builder.node(name).kind() == SyntaxKind::PrivateIdentifier {
            let node = self.builder.node(name);
            let (start, end) = (self.skip_trivia(node.pos() as usize), node.end() as usize);
            self.parse_error_at(
                start,
                end,
                diagnostics::AN_OPTIONAL_CHAIN_CANNOT_CONTAIN_PRIVATE_IDENTIFIERS,
                &[],
            );
        }
        if let Some((_, Some(type_arguments))) = self.expression_with_type_arguments(expression) {
            // Report from the `<` through the `>` of the type arguments.
            let start = type_arguments.pos() as usize - 1;
            let end = self.skip_trivia(type_arguments.end() as usize) + 1;
            self.parse_error_at(
                start,
                end,
                diagnostics::AN_INSTANTIATION_EXPRESSION_CANNOT_BE_FOLLOWED_BY_A_PROPERTY_ACCESS,
                &[],
            );
        }
        self.finish_node_with_flags(
            SyntaxKind::PropertyAccessExpression,
            pos,
            optional_chain_flags(is_optional_chain),
            NodeData::PropertyAccessExpression(PropertyAccessExpression {
                expression,
                question_dot_token,
                name,
            }),
        )
    }

    /// Returns whether `node` continues an optional chain, marking intervening non-null
    /// assertions as part of the chain.
    fn try_reparse_optional_chain(&mut self, node: NodeId) -> bool {
        if self
            .builder
            .node(node)
            .flags()
            .intersects(NodeFlags::OPTIONAL_CHAIN)
        {
            return true;
        }
        let Some(mut expression) = self.non_null_expression_operand(node) else {
            return false;
        };
        while let Some(inner) = self.non_null_expression_operand(expression) {
            if self
                .builder
                .node(expression)
                .flags()
                .intersects(NodeFlags::OPTIONAL_CHAIN)
            {
                break;
            }
            expression = inner;
        }
        if !self
            .builder
            .node(expression)
            .flags()
            .intersects(NodeFlags::OPTIONAL_CHAIN)
        {
            return false;
        }
        let mut current = node;
        while let Some(inner) = self.non_null_expression_operand(current) {
            self.builder.add_flags(current, NodeFlags::OPTIONAL_CHAIN);
            current = inner;
        }
        true
    }

    fn non_null_expression_operand(&self, node: NodeId) -> Option<NodeId> {
        self.builder
            .node(node)
            .data()
            .as_non_null_expression()
            .map(|data| data.expression)
    }

    fn parse_element_access_expression_rest(
        &mut self,
        pos: usize,
        expression: NodeId,
        question_dot_token: Option<NodeId>,
    ) -> NodeId {
        let argument_expression = if self.token == SyntaxKind::CloseBracketToken {
            let node_pos = self.node_pos();
            self.parse_error_at(
                node_pos,
                node_pos,
                diagnostics::AN_ELEMENT_ACCESS_EXPRESSION_SHOULD_TAKE_AN_ARGUMENT,
                &[],
            );
            self.create_missing_identifier()
        } else {
            self.parse_expression_allow_in()
        };
        self.parse_expected(SyntaxKind::CloseBracketToken);
        let is_optional_chain =
            question_dot_token.is_some() || self.try_reparse_optional_chain(expression);
        self.finish_node_with_flags(
            SyntaxKind::ElementAccessExpression,
            pos,
            optional_chain_flags(is_optional_chain),
            NodeData::ElementAccessExpression(ElementAccessExpression {
                expression,
                question_dot_token,
                argument_expression,
            }),
        )
    }

    pub(super) fn parse_call_expression_rest(
        &mut self,
        pos: usize,
        mut expression: NodeId,
    ) -> NodeId {
        loop {
            expression = self.parse_member_expression_rest(pos, expression, true);
            let question_dot_token = self.parse_optional_token(SyntaxKind::QuestionDotToken);
            let mut type_arguments = None;
            if question_dot_token.is_some() {
                type_arguments = self.try_parse_type_arguments_in_expression();
                if self.is_template_start_of_tagged_template() {
                    expression = self.parse_tagged_template_rest(
                        pos,
                        expression,
                        question_dot_token,
                        type_arguments,
                    );
                    continue;
                }
            }
            if type_arguments.is_some() || self.token == SyntaxKind::OpenParenToken {
                // Type arguments of an instantiation expression move onto the call.
                if question_dot_token.is_none()
                    && let Some((inner, inner_type_arguments)) =
                        self.expression_with_type_arguments(expression)
                {
                    type_arguments = inner_type_arguments;
                    expression = inner;
                }
                let arguments = self.parse_argument_list();
                let is_optional_chain =
                    question_dot_token.is_some() || self.try_reparse_optional_chain(expression);
                expression = self.finish_node_with_flags(
                    SyntaxKind::CallExpression,
                    pos,
                    optional_chain_flags(is_optional_chain),
                    NodeData::CallExpression(CallExpression {
                        expression,
                        question_dot_token,
                        type_arguments,
                        arguments,
                    }),
                );
                continue;
            }
            if let Some(question_dot_token) = question_dot_token {
                // `?.` followed by nothing usable: report a missing name.
                self.parse_error_at_current_token(diagnostics::IDENTIFIER_EXPECTED, &[]);
                let name = self.create_missing_identifier();
                expression = self.finish_node_with_flags(
                    SyntaxKind::PropertyAccessExpression,
                    pos,
                    NodeFlags::OPTIONAL_CHAIN,
                    NodeData::PropertyAccessExpression(PropertyAccessExpression {
                        expression,
                        question_dot_token: Some(question_dot_token),
                        name,
                    }),
                );
            }
            return expression;
        }
    }

    fn parse_argument_list(&mut self) -> NodeList {
        self.parse_expected(SyntaxKind::OpenParenToken);
        let result = self
            .parse_delimited_list(ParsingContext::ArgumentExpressions, |parser| {
                Some(parser.parse_argument_expression())
            })
            .expect("argument expressions always parse");
        self.parse_expected(SyntaxKind::CloseParenToken);
        result
    }

    fn parse_argument_expression(&mut self) -> NodeId {
        self.do_in_context(
            NodeFlags::DISALLOW_IN_CONTEXT | NodeFlags::DECORATOR_CONTEXT,
            false,
            Self::parse_argument_or_array_literal_element,
        )
    }

    fn parse_argument_or_array_literal_element(&mut self) -> NodeId {
        match self.token {
            SyntaxKind::DotDotDotToken => {
                let pos = self.node_pos();
                self.parse_expected(SyntaxKind::DotDotDotToken);
                let expression = self.parse_assignment_expression_or_higher();
                self.finish_node(
                    SyntaxKind::SpreadElement,
                    pos,
                    NodeData::SpreadElement(SpreadElement { expression }),
                )
            }
            SyntaxKind::CommaToken => {
                let pos = self.node_pos();
                self.finish_node(
                    SyntaxKind::OmittedExpression,
                    pos,
                    NodeData::OmittedExpression,
                )
            }
            _ => self.parse_assignment_expression_or_higher(),
        }
    }

    fn parse_tagged_template_rest(
        &mut self,
        pos: usize,
        tag: NodeId,
        question_dot_token: Option<NodeId>,
        type_arguments: Option<NodeList>,
    ) -> NodeId {
        let template = if self.token == SyntaxKind::NoSubstitutionTemplateLiteral {
            self.rescan_template_token(true);
            self.parse_literal_expression()
        } else {
            self.parse_template_expression(true)
        };
        let is_optional_chain = question_dot_token.is_some()
            || self
                .builder
                .node(tag)
                .flags()
                .intersects(NodeFlags::OPTIONAL_CHAIN);
        self.finish_node_with_flags(
            SyntaxKind::TaggedTemplateExpression,
            pos,
            optional_chain_flags(is_optional_chain),
            NodeData::TaggedTemplateExpression(TaggedTemplateExpression {
                tag,
                question_dot_token,
                type_arguments,
                template,
            }),
        )
    }

    fn parse_template_expression(&mut self, is_tagged_template: bool) -> NodeId {
        let pos = self.node_pos();
        let head = self.parse_template_head(is_tagged_template);
        let template_spans = self.parse_template_spans(is_tagged_template);
        self.finish_node(
            SyntaxKind::TemplateExpression,
            pos,
            NodeData::TemplateExpression(TemplateExpression {
                head,
                template_spans,
            }),
        )
    }

    fn parse_template_spans(&mut self, is_tagged_template: bool) -> NodeList {
        let pos = self.node_pos();
        let mut spans = Vec::new();
        loop {
            let span_pos = self.node_pos();
            let expression = self.parse_expression_allow_in();
            let literal = self.parse_literal_of_template_span(is_tagged_template);
            let span = self.finish_node(
                SyntaxKind::TemplateSpan,
                span_pos,
                NodeData::TemplateSpan(TemplateSpan {
                    expression,
                    literal,
                }),
            );
            spans.push(span);
            if self.builder.node(literal).kind() != SyntaxKind::TemplateMiddle {
                break;
            }
        }
        let end = self.node_pos();
        self.new_node_list(pos, end, spans)
    }

    pub(super) fn parse_template_head(&mut self, is_tagged_template: bool) -> NodeId {
        if !is_tagged_template
            && self
                .scanner
                .token_flags()
                .intersects(TokenFlags::IS_INVALID)
        {
            self.rescan_template_token(false);
        }
        let pos = self.node_pos();
        let data = NodeData::TemplateHead(TemplateHead {
            text: self.scanner.token_value().into(),
            raw_text: self.template_literal_raw_text(2).into(),
            template_flags: self.scanner.token_flags(),
        });
        self.next_token();
        self.finish_node(SyntaxKind::TemplateHead, pos, data)
    }

    fn template_literal_raw_text(&self, end_length: usize) -> &str {
        let text = self.scanner.token_text();
        let end_length = if self
            .scanner
            .token_flags()
            .intersects(TokenFlags::UNTERMINATED)
        {
            0
        } else {
            end_length
        };
        &text[1..text.len() - end_length]
    }

    pub(super) fn parse_literal_of_template_span(&mut self, is_tagged_template: bool) -> NodeId {
        if self.token == SyntaxKind::CloseBraceToken {
            self.rescan_template_token(is_tagged_template);
            return self.parse_template_middle_or_tail();
        }
        self.parse_error_at_current_token(
            diagnostics::X_0_EXPECTED,
            &[token_to_string(SyntaxKind::CloseBraceToken)],
        );
        let pos = self.node_pos();
        self.finish_node(
            SyntaxKind::TemplateTail,
            pos,
            NodeData::TemplateTail(TemplateTail {
                text: "".into(),
                raw_text: "".into(),
                template_flags: TokenFlags::NONE,
            }),
        )
    }

    fn parse_template_middle_or_tail(&mut self) -> NodeId {
        let pos = self.node_pos();
        let kind = self.token;
        let text = self.scanner.token_value().into();
        let template_flags = self.scanner.token_flags();
        let data = if kind == SyntaxKind::TemplateMiddle {
            NodeData::TemplateMiddle(TemplateMiddle {
                text,
                raw_text: self.template_literal_raw_text(2).into(),
                template_flags,
            })
        } else {
            NodeData::TemplateTail(TemplateTail {
                text,
                raw_text: self.template_literal_raw_text(1).into(),
                template_flags,
            })
        };
        let kind = if kind == SyntaxKind::TemplateMiddle {
            SyntaxKind::TemplateMiddle
        } else {
            SyntaxKind::TemplateTail
        };
        self.next_token();
        self.finish_node(kind, pos, data)
    }

    fn parse_primary_expression(&mut self) -> NodeId {
        let token = self.token;
        match token {
            SyntaxKind::NoSubstitutionTemplateLiteral => {
                if self
                    .scanner
                    .token_flags()
                    .intersects(TokenFlags::IS_INVALID)
                {
                    self.rescan_template_token(false);
                }
                self.parse_literal_expression()
            }
            SyntaxKind::NumericLiteral | SyntaxKind::BigIntLiteral | SyntaxKind::StringLiteral => {
                self.parse_literal_expression()
            }
            SyntaxKind::ThisKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword => self.parse_keyword_expression(),
            SyntaxKind::OpenParenToken => self.parse_parenthesized_expression(),
            SyntaxKind::OpenBracketToken => self.parse_array_literal_expression(),
            SyntaxKind::OpenBraceToken => self.parse_object_literal_expression(),
            SyntaxKind::AsyncKeyword
                if self.look_ahead(Self::next_token_is_function_keyword_on_same_line) =>
            {
                self.parse_function_expression()
            }
            SyntaxKind::FunctionKeyword => self.parse_function_expression(),
            SyntaxKind::AtToken => self.parse_decorated_expression(),
            SyntaxKind::ClassKeyword => self.parse_class_expression(),
            SyntaxKind::NewKeyword => self.parse_new_expression_or_new_dot_target(),
            SyntaxKind::SlashToken | SyntaxKind::SlashEqualsToken
                if self.rescan_slash_token() == SyntaxKind::RegularExpressionLiteral =>
            {
                self.parse_literal_expression()
            }
            SyntaxKind::TemplateHead => self.parse_template_expression(false),
            SyntaxKind::PrivateIdentifier => self.parse_private_identifier(),
            _ => {
                self.parse_identifier_with_diagnostic(Some(diagnostics::EXPRESSION_EXPECTED), None)
            }
        }
    }

    fn parse_parenthesized_expression(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        self.parse_expected(SyntaxKind::OpenParenToken);
        let expression = self.parse_expression_allow_in();
        self.parse_expected(SyntaxKind::CloseParenToken);
        let result = self.finish_node(
            SyntaxKind::ParenthesizedExpression,
            pos,
            NodeData::ParenthesizedExpression(ParenthesizedExpression { expression }),
        );
        self.with_jsdoc(result, jsdoc);
        result
    }

    fn parse_array_literal_expression(&mut self) -> NodeId {
        let pos = self.node_pos();
        let open_bracket_position = self.scanner.token_start();
        let open_bracket_parsed = self.parse_expected(SyntaxKind::OpenBracketToken);
        let multi_line = self.has_preceding_line_break();
        let elements = self
            .parse_delimited_list(ParsingContext::ArrayLiteralMembers, |parser| {
                Some(parser.parse_argument_or_array_literal_element())
            })
            .expect("array literal elements always parse");
        self.parse_expected_matching_brackets(
            SyntaxKind::OpenBracketToken,
            SyntaxKind::CloseBracketToken,
            open_bracket_parsed,
            open_bracket_position,
        );
        self.finish_node(
            SyntaxKind::ArrayLiteralExpression,
            pos,
            NodeData::ArrayLiteralExpression(ArrayLiteralExpression {
                elements,
                multi_line,
            }),
        )
    }

    fn parse_object_literal_expression(&mut self) -> NodeId {
        let pos = self.node_pos();
        let open_brace_position = self.scanner.token_start();
        let open_brace_parsed = self.parse_expected(SyntaxKind::OpenBraceToken);
        let multi_line = self.has_preceding_line_break();
        let properties = self
            .parse_delimited_list(ParsingContext::ObjectLiteralMembers, |parser| {
                Some(parser.parse_object_literal_element())
            })
            .expect("object literal members always parse");
        self.parse_expected_matching_brackets(
            SyntaxKind::OpenBraceToken,
            SyntaxKind::CloseBraceToken,
            open_brace_parsed,
            open_brace_position,
        );
        self.finish_node(
            SyntaxKind::ObjectLiteralExpression,
            pos,
            NodeData::ObjectLiteralExpression(ObjectLiteralExpression {
                properties,
                multi_line,
            }),
        )
    }

    fn parse_object_literal_element(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        if self.parse_optional(SyntaxKind::DotDotDotToken) {
            let expression = self.parse_assignment_expression_or_higher();
            let result = self.finish_node(
                SyntaxKind::SpreadAssignment,
                pos,
                NodeData::SpreadAssignment(SpreadAssignment { expression }),
            );
            self.with_jsdoc(result, jsdoc);
            return result;
        }
        let modifiers = self.parse_modifiers_with(ModifierOptions::DECORATED);
        if self.parse_contextual_modifier(SyntaxKind::GetKeyword) {
            return self.parse_accessor_declaration(
                pos,
                jsdoc,
                modifiers,
                SyntaxKind::GetAccessor,
                SignatureFlags::default(),
            );
        }
        if self.parse_contextual_modifier(SyntaxKind::SetKeyword) {
            return self.parse_accessor_declaration(
                pos,
                jsdoc,
                modifiers,
                SyntaxKind::SetAccessor,
                SignatureFlags::default(),
            );
        }
        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);
        let token_is_identifier = self.is_identifier();
        let name = self.parse_property_name();
        let mut postfix_token = self.parse_optional_token(SyntaxKind::QuestionToken);
        if postfix_token.is_none() {
            postfix_token = self.parse_optional_token(SyntaxKind::ExclamationToken);
        }
        if asterisk_token.is_some()
            || matches!(
                self.token,
                SyntaxKind::OpenParenToken | SyntaxKind::LessThanToken
            )
        {
            return self.parse_method_declaration(
                pos,
                jsdoc,
                modifiers,
                asterisk_token,
                name,
                postfix_token,
                None,
            );
        }
        // `{ a = 1 }` is a CoverInitializedName, which object assignment patterns need.
        let (kind, data) = if token_is_identifier && self.token != SyntaxKind::ColonToken {
            let equals_token = self.parse_optional_token(SyntaxKind::EqualsToken);
            let object_assignment_initializer = equals_token.map(|_| {
                self.do_in_context(
                    NodeFlags::DISALLOW_IN_CONTEXT,
                    false,
                    Self::parse_assignment_expression_or_higher,
                )
            });
            (
                SyntaxKind::ShorthandPropertyAssignment,
                NodeData::ShorthandPropertyAssignment(ShorthandPropertyAssignment {
                    modifiers,
                    name,
                    postfix_token,
                    type_node: None,
                    equals_token,
                    object_assignment_initializer,
                }),
            )
        } else {
            self.parse_expected(SyntaxKind::ColonToken);
            let initializer = self.do_in_context(
                NodeFlags::DISALLOW_IN_CONTEXT,
                false,
                Self::parse_assignment_expression_or_higher,
            );
            (
                SyntaxKind::PropertyAssignment,
                NodeData::PropertyAssignment(PropertyAssignment {
                    modifiers,
                    name,
                    postfix_token,
                    type_node: None,
                    initializer,
                }),
            )
        };
        let node = self.finish_node(kind, pos, data);
        self.with_jsdoc(node, jsdoc);
        node
    }

    fn parse_new_expression_or_new_dot_target(&mut self) -> NodeId {
        let pos = self.node_pos();
        self.parse_expected(SyntaxKind::NewKeyword);
        if self.parse_optional(SyntaxKind::DotToken) {
            let name = self.parse_identifier_name();
            return self.finish_node(
                SyntaxKind::MetaProperty,
                pos,
                NodeData::MetaProperty(MetaProperty {
                    keyword_token: SyntaxKind::NewKeyword,
                    name,
                }),
            );
        }
        let expression_pos = self.node_pos();
        let primary = self.parse_primary_expression();
        let mut expression = self.parse_member_expression_rest(expression_pos, primary, false);
        // Type arguments of an instantiation expression move onto the `new` expression.
        let mut type_arguments = None;
        if let Some((inner, inner_type_arguments)) = self.expression_with_type_arguments(expression)
        {
            type_arguments = inner_type_arguments;
            expression = inner;
        }
        if self.token == SyntaxKind::QuestionDotToken {
            let node = self.builder.node(expression);
            let start = self.skip_trivia(node.pos() as usize);
            let text = self.scanner.text()[start..node.end() as usize].to_owned();
            self.parse_error_at_current_token(
                diagnostics::INVALID_OPTIONAL_CHAIN_FROM_NEW_EXPRESSION_DID_YOU_MEAN_TO_CALL_0,
                &[&text],
            );
        }
        let arguments =
            (self.token == SyntaxKind::OpenParenToken).then(|| self.parse_argument_list());
        self.finish_node(
            SyntaxKind::NewExpression,
            pos,
            NodeData::NewExpression(NewExpression {
                expression,
                type_arguments,
                arguments,
            }),
        )
    }

    /// Returns an instantiation expression's operand and type arguments.
    fn expression_with_type_arguments(
        &self,
        expression: NodeId,
    ) -> Option<(NodeId, Option<NodeList>)> {
        self.builder
            .node(expression)
            .data()
            .as_expression_with_type_arguments()
            .map(|data| (data.expression, data.type_arguments))
    }

    fn finish_expression_with_type_arguments(
        &mut self,
        pos: usize,
        expression: NodeId,
        type_arguments: NodeList,
    ) -> NodeId {
        self.finish_node(
            SyntaxKind::ExpressionWithTypeArguments,
            pos,
            NodeData::ExpressionWithTypeArguments(ExpressionWithTypeArguments {
                expression,
                type_arguments: Some(type_arguments),
            }),
        )
    }

    /// Parses `<...>` as type arguments when the tokens that follow favor that reading over a
    /// relational expression. JavaScript files never have type arguments.
    fn try_parse_type_arguments_in_expression(&mut self) -> Option<NodeList> {
        if self.in_context(NodeFlags::JAVA_SCRIPT_FILE)
            || !matches!(
                self.token,
                SyntaxKind::LessThanToken | SyntaxKind::LessThanLessThanToken
            )
        {
            return None;
        }
        let state = self.mark();
        if self.rescan_less_than_token() == SyntaxKind::LessThanToken {
            self.next_token();
            let type_arguments = self
                .parse_delimited_list(ParsingContext::TypeArguments, |parser| {
                    Some(parser.parse_type())
                });
            if self.rescan_greater_than_token() == SyntaxKind::GreaterThanToken {
                self.next_token();
                if self.can_follow_type_arguments_in_expression() {
                    return type_arguments;
                }
            }
        }
        self.rewind(state);
        None
    }

    fn can_follow_type_arguments_in_expression(&mut self) -> bool {
        match self.token {
            // `f<T>(`, `f<T>`...``, and `f<T>`...${`.
            SyntaxKind::OpenParenToken
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::TemplateHead => true,
            // `<` never follows type arguments, `>` is ambiguous with a rescanned `>>`, and `+`
            // and `-` are unary here.
            SyntaxKind::LessThanToken
            | SyntaxKind::GreaterThanToken
            | SyntaxKind::PlusToken
            | SyntaxKind::MinusToken => false,
            _ => {
                self.has_preceding_line_break()
                    || self.is_binary_operator()
                    || !self.is_start_of_expression()
            }
        }
    }

    fn parse_function_expression(&mut self) -> NodeId {
        let saved_context_flags = self.context_flags;
        self.set_context_flags(NodeFlags::DECORATOR_CONTEXT, false);
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        let modifiers = self.parse_modifiers();
        self.parse_expected(SyntaxKind::FunctionKeyword);
        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);
        let is_generator = asterisk_token.is_some();
        let is_async =
            modifiers.is_some_and(|modifiers| modifiers.flags().intersects(ModifierFlags::ASYNC));
        let mut name_context = NodeFlags::NONE;
        if is_generator {
            name_context |= NodeFlags::YIELD_CONTEXT;
        }
        if is_async {
            name_context |= NodeFlags::AWAIT_CONTEXT;
        }
        let name = if name_context == NodeFlags::NONE {
            self.parse_optional_binding_identifier()
        } else {
            self.do_in_context(name_context, true, Self::parse_optional_binding_identifier)
        };
        let flags = SignatureFlags::function(is_generator, is_async);
        let type_parameters = self.parse_type_parameters();
        let parameters = self.parse_parameters(flags);
        let type_node = self.parse_return_type(SyntaxKind::ColonToken, false);
        let body = self.parse_function_block(flags, None);
        self.context_flags = saved_context_flags;
        let result = self.finish_node(
            SyntaxKind::FunctionExpression,
            pos,
            NodeData::FunctionExpression(FunctionExpression {
                modifiers,
                asterisk_token,
                name,
                type_parameters,
                parameters,
                type_node,
                full_signature: None,
                body,
            }),
        );
        self.with_jsdoc(result, jsdoc);
        result
    }

    fn next_token_is_identifier_or_keyword_or_literal_on_same_line(&mut self) -> bool {
        self.next_token();
        (self.token.is_identifier_or_keyword()
            || matches!(
                self.token,
                SyntaxKind::NumericLiteral | SyntaxKind::BigIntLiteral | SyntaxKind::StringLiteral
            ))
            && !self.has_preceding_line_break()
    }

    pub(super) fn next_token_is_function_keyword_on_same_line(&mut self) -> bool {
        self.next_token() == SyntaxKind::FunctionKeyword && !self.has_preceding_line_break()
    }

    pub(super) fn identifier_text(&self, id: NodeId) -> &str {
        self.builder
            .node(id)
            .data()
            .as_identifier()
            .map_or("", |identifier| &identifier.text)
    }

    /// Returns whether `expression` is a `LeftHandSideExpression` in TypeScript-Go's sense.
    fn is_left_hand_side_expression(&self, expression: NodeId) -> bool {
        is_left_hand_side_expression_kind(self.builder.node(expression).kind())
    }
}

fn optional_chain_flags(is_optional_chain: bool) -> NodeFlags {
    if is_optional_chain {
        NodeFlags::OPTIONAL_CHAIN
    } else {
        NodeFlags::NONE
    }
}

/// Mirrors TypeScript-Go's `ast.IsLeftHandSideExpressionKind`.
fn is_left_hand_side_expression_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::PropertyAccessExpression
            | SyntaxKind::ElementAccessExpression
            | SyntaxKind::NewExpression
            | SyntaxKind::CallExpression
            | SyntaxKind::JsxElement
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxFragment
            | SyntaxKind::TaggedTemplateExpression
            | SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::ClassExpression
            | SyntaxKind::FunctionExpression
            | SyntaxKind::Identifier
            | SyntaxKind::PrivateIdentifier
            | SyntaxKind::RegularExpressionLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::StringLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::TemplateExpression
            | SyntaxKind::FalseKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::ThisKeyword
            | SyntaxKind::TrueKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::NonNullExpression
            | SyntaxKind::ExpressionWithTypeArguments
            | SyntaxKind::MetaProperty
            | SyntaxKind::ImportKeyword
            | SyntaxKind::MissingDeclaration
    )
}
