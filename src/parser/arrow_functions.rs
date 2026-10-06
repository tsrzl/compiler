//! Arrow functions, including speculative recognition of parenthesized parameter lists.

use crate::ast::{
    ArrowFunction, ModifierFlags, ModifierList, NodeData, NodeFlags, NodeId, NodeList, SyntaxKind,
};
use crate::scanner::LanguageVariant;

use super::signatures::SignatureFlags;
use super::{JsdocScannerInfo, Parser};

/// Whether the tokens ahead start a parenthesized arrow function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tristate {
    False,
    True,
    Unknown,
}

impl Parser<'_> {
    pub(super) fn try_parse_parenthesized_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Option<NodeId> {
        match self.is_parenthesized_arrow_function_expression() {
            Tristate::False => None,
            // A definite arrow function is parsed without requiring a following `=>` or `{`.
            Tristate::True => self.parse_parenthesized_arrow_function_expression(true, true),
            Tristate::Unknown => {
                let state = self.mark();
                let result = self.parse_possible_parenthesized_arrow_function_expression(
                    allow_return_type_in_arrow_function,
                );
                if result.is_none() {
                    self.rewind(state);
                }
                result
            }
        }
    }

    fn is_parenthesized_arrow_function_expression(&mut self) -> Tristate {
        match self.token {
            SyntaxKind::OpenParenToken | SyntaxKind::LessThanToken | SyntaxKind::AsyncKeyword => {
                let state = self.mark();
                let result = self.next_is_parenthesized_arrow_function_expression();
                self.rewind(state);
                result
            }
            // A standalone `=>` was most likely meant to start an arrow function.
            SyntaxKind::EqualsGreaterThanToken => Tristate::True,
            _ => Tristate::False,
        }
    }

    fn next_is_parenthesized_arrow_function_expression(&mut self) -> Tristate {
        if self.token == SyntaxKind::AsyncKeyword {
            self.next_token();
            if self.has_preceding_line_break()
                || !matches!(
                    self.token,
                    SyntaxKind::OpenParenToken | SyntaxKind::LessThanToken
                )
            {
                return Tristate::False;
            }
        }
        let first = self.token;
        let second = self.next_token();
        if first == SyntaxKind::OpenParenToken {
            return self.classify_after_open_paren(second);
        }
        assert_eq!(
            first,
            SyntaxKind::LessThanToken,
            "an arrow function starts with `(` or `<`"
        );
        // `<` not followed by an identifier is not an arrow function.
        if !self.is_identifier() && self.token != SyntaxKind::ConstKeyword {
            return Tristate::False;
        }
        if self.language_variant != LanguageVariant::Jsx {
            return Tristate::Unknown;
        }
        // In JSX, `<T,` `<T =` and `<T extends U` (not followed by `=`, `>`, or `/`) are type
        // parameters; anything else starts an element.
        let is_arrow_function_in_jsx = self.look_ahead(|parser| {
            parser.parse_optional(SyntaxKind::ConstKeyword);
            match parser.next_token() {
                SyntaxKind::ExtendsKeyword => !matches!(
                    parser.next_token(),
                    SyntaxKind::EqualsToken | SyntaxKind::GreaterThanToken | SyntaxKind::SlashToken
                ),
                SyntaxKind::CommaToken | SyntaxKind::EqualsToken => true,
                _ => false,
            }
        });
        if is_arrow_function_in_jsx {
            Tristate::True
        } else {
            Tristate::False
        }
    }

    fn classify_after_open_paren(&mut self, second: SyntaxKind) -> Tristate {
        match second {
            // `() =>`, `():`, and `() {` (the last for recovery).
            SyntaxKind::CloseParenToken => match self.next_token() {
                SyntaxKind::EqualsGreaterThanToken
                | SyntaxKind::ColonToken
                | SyntaxKind::OpenBraceToken => Tristate::True,
                _ => Tristate::False,
            },
            // `([` and `({` may start binding patterns or expressions.
            SyntaxKind::OpenBracketToken | SyntaxKind::OpenBraceToken => Tristate::Unknown,
            SyntaxKind::DotDotDotToken => Tristate::True,
            _ => self.classify_parameter_after_open_paren(second),
        }
    }

    fn classify_parameter_after_open_paren(&mut self, second: SyntaxKind) -> Tristate {
        // `(modifier identifier` is invalid but treated as an arrow function for a good error.
        if second.is_modifier_kind()
            && second != SyntaxKind::AsyncKeyword
            && self.look_ahead(|parser| {
                parser.next_token();
                parser.is_identifier()
            })
        {
            return if self.next_token() == SyntaxKind::AsKeyword {
                Tristate::False
            } else {
                Tristate::True
            };
        }
        // `this` is not a valid parameter name, but is parsed and reported semantically.
        if !self.is_identifier() && second != SyntaxKind::ThisKeyword {
            return Tristate::False;
        }
        match self.next_token() {
            // `(a:` is a type-annotated parameter.
            SyntaxKind::ColonToken => Tristate::True,
            SyntaxKind::QuestionToken => {
                self.next_token();
                if matches!(
                    self.token,
                    SyntaxKind::ColonToken
                        | SyntaxKind::CommaToken
                        | SyntaxKind::EqualsToken
                        | SyntaxKind::CloseParenToken
                ) {
                    Tristate::True
                } else {
                    Tristate::False
                }
            }
            SyntaxKind::CommaToken | SyntaxKind::EqualsToken | SyntaxKind::CloseParenToken => {
                Tristate::Unknown
            }
            _ => Tristate::False,
        }
    }

    fn parse_possible_parenthesized_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Option<NodeId> {
        let token_pos = self.scanner.token_start();
        if self.not_parenthesized_arrow.contains(&token_pos) {
            return None;
        }
        let result = self.parse_parenthesized_arrow_function_expression(
            false,
            allow_return_type_in_arrow_function,
        );
        if result.is_none() {
            self.not_parenthesized_arrow.insert(token_pos);
        }
        result
    }

    /// Parses a parenthesized arrow function. Without ambiguity allowed, returns `None` as soon
    /// as the tokens could instead be an ordinary expression.
    fn parse_parenthesized_arrow_function_expression(
        &mut self,
        allow_ambiguity: bool,
        allow_return_type_in_arrow_function: bool,
    ) -> Option<NodeId> {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        let modifiers = self.parse_modifiers_for_arrow_function();
        let is_async =
            modifiers.is_some_and(|modifiers| modifiers.flags().intersects(ModifierFlags::ASYNC));
        let flags = SignatureFlags::function(false, is_async);
        // A speculative signature must be complete; otherwise `a => (b => c)` would read `(b =>`
        // as an arrow function missing its `)`.
        let type_parameters = self.parse_type_parameters();
        let parameters = self.parse_arrow_parameters(flags, allow_ambiguity)?;
        let has_return_colon = self.token == SyntaxKind::ColonToken;
        let type_node = self.parse_return_type(SyntaxKind::ColonToken, false);
        if let Some(type_node) = type_node
            && !allow_ambiguity
            && self.type_has_arrow_function_blocking_parse_error(type_node)
        {
            return None;
        }
        // A signature alone may be another expression, such as `(x = 10)` or `a ? (b): c`.
        if !allow_ambiguity
            && !matches!(
                self.token,
                SyntaxKind::EqualsGreaterThanToken | SyntaxKind::OpenBraceToken
            )
        {
            return None;
        }
        let last_token = self.token;
        let equals_greater_than_token =
            self.parse_expected_token(SyntaxKind::EqualsGreaterThanToken);
        let body = if matches!(
            last_token,
            SyntaxKind::EqualsGreaterThanToken | SyntaxKind::OpenBraceToken
        ) {
            self.parse_arrow_function_expression_body(is_async, allow_return_type_in_arrow_function)
        } else {
            self.parse_identifier()
        };
        // In the true branch of a conditional, a return-type colon is only allowed when another
        // colon follows to end the conditional, as in `a ? (x): string => x : null`.
        if !allow_return_type_in_arrow_function
            && has_return_colon
            && self.token != SyntaxKind::ColonToken
        {
            return None;
        }
        Some(self.finish_arrow_function(
            pos,
            jsdoc,
            modifiers,
            type_parameters,
            parameters,
            type_node,
            equals_greater_than_token,
            body,
        ))
    }

    fn parse_arrow_parameters(
        &mut self,
        flags: SignatureFlags,
        allow_ambiguity: bool,
    ) -> Option<NodeList> {
        if !self.parse_expected(SyntaxKind::OpenParenToken) {
            if !allow_ambiguity {
                return None;
            }
            let pos = self.node_pos();
            return Some(self.builder.add_missing_list(super::to_u32(pos)));
        }
        let parameters = self.parse_parameters_worker(flags, allow_ambiguity)?;
        if !self.parse_expected(SyntaxKind::CloseParenToken) && !allow_ambiguity {
            return None;
        }
        Some(parameters)
    }

    /// Returns whether a speculatively parsed return type shows the tokens were not a signature.
    fn type_has_arrow_function_blocking_parse_error(&self, node: NodeId) -> bool {
        let node = self.builder.node(node);
        match node.data() {
            NodeData::TypeReferenceNode(reference) => !self.node_is_present(reference.type_name),
            NodeData::FunctionTypeNode(function) => {
                function.parameters.is_missing()
                    || function.type_node.is_some_and(|type_node| {
                        self.type_has_arrow_function_blocking_parse_error(type_node)
                    })
            }
            NodeData::ConstructorTypeNode(constructor) => {
                constructor.parameters.is_missing()
                    || constructor.type_node.is_some_and(|type_node| {
                        self.type_has_arrow_function_blocking_parse_error(type_node)
                    })
            }
            NodeData::ParenthesizedTypeNode(parenthesized) => {
                self.type_has_arrow_function_blocking_parse_error(parenthesized.type_node)
            }
            _ => false,
        }
    }

    fn parse_modifiers_for_arrow_function(&mut self) -> Option<ModifierList> {
        if self.token != SyntaxKind::AsyncKeyword {
            return None;
        }
        let pos = self.node_pos();
        self.next_token();
        let modifier = self.finish_node(SyntaxKind::AsyncKeyword, pos, NodeData::Token);
        let end = self.builder.node(modifier).end() as usize;
        Some(self.new_modifier_list(pos, end, vec![modifier]))
    }

    fn parse_arrow_function_expression_body(
        &mut self,
        is_async: bool,
        allow_return_type_in_arrow_function: bool,
    ) -> NodeId {
        let flags = SignatureFlags::function(false, is_async);
        if self.token == SyntaxKind::OpenBraceToken {
            return self.parse_function_block(flags, None);
        }
        // A declaration statement where the body belongs suggests a missing `{`; parse a block
        // so that the next `}` does not close the enclosing construct.
        if !matches!(
            self.token,
            SyntaxKind::SemicolonToken | SyntaxKind::FunctionKeyword | SyntaxKind::ClassKeyword
        ) && self.is_start_of_statement()
            && !self.is_start_of_expression_statement()
        {
            return self
                .parse_function_block(flags | SignatureFlags::IGNORE_MISSING_OPEN_BRACE, None);
        }
        let saved_context_flags = self.context_flags;
        self.set_context_flags(NodeFlags::AWAIT_CONTEXT, is_async);
        self.set_context_flags(NodeFlags::YIELD_CONTEXT, false);
        let body =
            self.parse_assignment_expression_or_higher_worker(allow_return_type_in_arrow_function);
        self.context_flags = saved_context_flags;
        body
    }

    fn is_start_of_expression_statement(&mut self) -> bool {
        !matches!(
            self.token,
            SyntaxKind::OpenBraceToken
                | SyntaxKind::FunctionKeyword
                | SyntaxKind::ClassKeyword
                | SyntaxKind::AtToken
        ) && self.is_start_of_expression()
    }

    pub(super) fn try_parse_async_simple_arrow_function_expression(
        &mut self,
        allow_return_type_in_arrow_function: bool,
    ) -> Option<NodeId> {
        if self.token != SyntaxKind::AsyncKeyword
            || !self.look_ahead(Self::next_is_unparenthesized_async_arrow_function)
        {
            return None;
        }
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        let async_modifier = self.parse_modifiers_for_arrow_function();
        let parameter =
            self.parse_binary_expression_or_higher(crate::ast::OperatorPrecedence::LOWEST);
        Some(self.parse_simple_arrow_function_expression(
            pos,
            parameter,
            allow_return_type_in_arrow_function,
            jsdoc,
            async_modifier,
        ))
    }

    fn next_is_unparenthesized_async_arrow_function(&mut self) -> bool {
        self.next_token();
        // `async =>` is a simple arrow function whose parameter is named `async`.
        if self.has_preceding_line_break() || self.token == SyntaxKind::EqualsGreaterThanToken {
            return false;
        }
        let expression =
            self.parse_binary_expression_or_higher(crate::ast::OperatorPrecedence::LOWEST);
        !self.has_preceding_line_break()
            && self.builder.node(expression).kind() == SyntaxKind::Identifier
            && self.token == SyntaxKind::EqualsGreaterThanToken
    }

    pub(super) fn parse_simple_arrow_function_expression(
        &mut self,
        pos: usize,
        identifier: NodeId,
        allow_return_type_in_arrow_function: bool,
        jsdoc: JsdocScannerInfo,
        async_modifier: Option<ModifierList>,
    ) -> NodeId {
        assert_eq!(
            self.token,
            SyntaxKind::EqualsGreaterThanToken,
            "a simple arrow function continues at `=>`"
        );
        let identifier_node = self.builder.node(identifier);
        let (parameter_pos, parameter_end) = (
            identifier_node.pos() as usize,
            identifier_node.end() as usize,
        );
        let parameter =
            self.finish_parameter(parameter_pos, None, None, identifier, None, None, None);
        let parameters = self.new_node_list(parameter_pos, parameter_end, vec![parameter]);
        let equals_greater_than_token =
            self.parse_expected_token(SyntaxKind::EqualsGreaterThanToken);
        let body = self.parse_arrow_function_expression_body(
            async_modifier.is_some(),
            allow_return_type_in_arrow_function,
        );
        self.finish_arrow_function(
            pos,
            jsdoc,
            async_modifier,
            None,
            parameters,
            None,
            equals_greater_than_token,
            body,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_arrow_function(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
        type_parameters: Option<NodeList>,
        parameters: NodeList,
        type_node: Option<NodeId>,
        equals_greater_than_token: NodeId,
        body: NodeId,
    ) -> NodeId {
        let result = self.finish_node(
            SyntaxKind::ArrowFunction,
            pos,
            NodeData::ArrowFunction(ArrowFunction {
                modifiers,
                type_parameters,
                parameters,
                type_node,
                full_signature: None,
                equals_greater_than_token,
                body,
            }),
        );
        self.with_jsdoc(result, jsdoc);
        result
    }
}
