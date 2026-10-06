//! Type parameters, parameters, return types, and function bodies.

use crate::ast::{
    ModifierList, NodeData, NodeFlags, NodeId, NodeList, ParameterDeclaration, SyntaxKind,
    TypeParameterDeclaration, TypePredicateNode,
};
use crate::diagnostics::{self, Message};

use super::modifiers::ModifierOptions;
use super::{Parser, ParsingContext, token_to_string};

crate::ast::flags_type! {
    /// The context a signature is parsed in, matching TypeScript-Go's `ParseFlags`.
    SignatureFlags {
        NONE = 0;
        /// The body is in a [Yield] context.
        YIELD = 1 << 0;
        /// The body is in an [Await] context.
        AWAIT = 1 << 1;
        /// The signature belongs to a type, such as a method signature or function type.
        TYPE = 1 << 2;
        /// A missing `{` before the body is tolerated for error recovery.
        IGNORE_MISSING_OPEN_BRACE = 1 << 4;
    }
}

impl SignatureFlags {
    /// Returns the flags for a function body with the given generator and async modifiers.
    pub(super) fn function(is_generator: bool, is_async: bool) -> Self {
        let mut flags = Self::NONE;
        if is_generator {
            flags |= Self::YIELD;
        }
        if is_async {
            flags |= Self::AWAIT;
        }
        flags
    }
}

impl Parser<'_> {
    pub(super) fn parse_type_parameters(&mut self) -> Option<NodeList> {
        if self.token != SyntaxKind::LessThanToken {
            return None;
        }
        self.parse_bracketed_list(
            ParsingContext::TypeParameters,
            |parser| Some(parser.parse_type_parameter()),
            SyntaxKind::LessThanToken,
            SyntaxKind::GreaterThanToken,
        )
    }

    fn parse_type_parameter(&mut self) -> NodeId {
        let pos = self.node_pos();
        let modifiers = self.parse_modifiers_with(ModifierOptions {
            permit_const_as_modifier: true,
            ..ModifierOptions::default()
        });
        let name = self.parse_identifier();
        let mut constraint = None;
        let mut expression = None;
        if self.parse_optional(SyntaxKind::ExtendsKeyword) {
            // An expression where a constraint type belongs is parsed for recovery; the unary
            // level keeps `<T extends "">` from consuming the `>`.
            if self.is_start_of_type(false) || !self.is_start_of_expression() {
                constraint = Some(self.parse_type());
            } else {
                expression = Some(self.parse_unary_expression_or_higher());
            }
        }
        let default_type = self
            .parse_optional(SyntaxKind::EqualsToken)
            .then(|| self.parse_type());
        self.finish_node(
            SyntaxKind::TypeParameter,
            pos,
            NodeData::TypeParameterDeclaration(TypeParameterDeclaration {
                modifiers,
                name,
                constraint,
                expression,
                default_type,
            }),
        )
    }

    pub(super) fn parse_parameters(&mut self, flags: SignatureFlags) -> NodeList {
        if self.parse_expected(SyntaxKind::OpenParenToken) {
            let parameters = self
                .parse_parameters_worker(flags, true)
                .expect("unambiguous parameter lists always parse");
            self.parse_expected(SyntaxKind::CloseParenToken);
            return parameters;
        }
        let pos = self.node_pos();
        self.builder.add_missing_list(super::to_u32(pos))
    }

    /// Parses parameters in the signature's context. Without ambiguity allowed, returns `None` at
    /// the first parameter that cannot start a binding name.
    pub(super) fn parse_parameters_worker(
        &mut self,
        flags: SignatureFlags,
        allow_ambiguity: bool,
    ) -> Option<NodeList> {
        let in_await_context = self.in_context(NodeFlags::AWAIT_CONTEXT);
        let saved_context_flags = self.context_flags;
        self.set_context_flags(
            NodeFlags::YIELD_CONTEXT,
            flags.contains(SignatureFlags::YIELD),
        );
        self.set_context_flags(
            NodeFlags::AWAIT_CONTEXT,
            flags.contains(SignatureFlags::AWAIT),
        );
        let parameters = self.parse_delimited_list(ParsingContext::Parameters, |parser| {
            parser.parse_parameter_with(in_await_context, allow_ambiguity)
        });
        self.context_flags = saved_context_flags;
        parameters
    }

    pub(super) fn parse_parameter(&mut self) -> NodeId {
        self.parse_parameter_with(false, true)
            .expect("ambiguous parameters always parse")
    }

    fn parse_parameter_with(
        &mut self,
        in_outer_await_context: bool,
        allow_ambiguity: bool,
    ) -> Option<NodeId> {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        // Decorators are parsed in the outer [Await] context; the rest in the function's.
        let saved_context_flags = self.context_flags;
        self.set_context_flags(NodeFlags::AWAIT_CONTEXT, in_outer_await_context);
        let modifiers = self.parse_modifiers_with(ModifierOptions::DECORATED);
        self.context_flags = saved_context_flags;
        if self.token == SyntaxKind::ThisKeyword {
            let name = self.create_identifier(true);
            let type_node = self.parse_type_annotation();
            if let Some(modifiers) = &modifiers {
                let first = self.builder.list(*modifiers.list())[0];
                let node = self.builder.node(first);
                let (start, end) = (node.pos() as usize, node.end() as usize);
                self.parse_error_at(
                    start,
                    end,
                    diagnostics::NEITHER_DECORATORS_NOR_MODIFIERS_MAY_BE_APPLIED_TO_THIS_PARAMETERS,
                    &[],
                );
            }
            let result = self.finish_parameter(pos, modifiers, None, name, None, type_node, None);
            self.with_jsdoc(result, jsdoc);
            return Some(result);
        }
        let dot_dot_dot_token = self.parse_optional_token(SyntaxKind::DotDotDotToken);
        if !allow_ambiguity && !self.is_parameter_name_start() {
            return None;
        }
        let name = self.parse_name_of_parameter(modifiers.as_ref());
        let question_token = self.parse_optional_token(SyntaxKind::QuestionToken);
        let type_node = self.parse_type_annotation();
        let initializer = self.parse_initializer();
        let result = self.finish_parameter(
            pos,
            modifiers,
            dot_dot_dot_token,
            name,
            question_token,
            type_node,
            initializer,
        );
        self.with_jsdoc(result, jsdoc);
        Some(result)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn finish_parameter(
        &mut self,
        pos: usize,
        modifiers: Option<ModifierList>,
        dot_dot_dot_token: Option<NodeId>,
        name: NodeId,
        question_token: Option<NodeId>,
        type_node: Option<NodeId>,
        initializer: Option<NodeId>,
    ) -> NodeId {
        self.finish_node(
            SyntaxKind::Parameter,
            pos,
            NodeData::ParameterDeclaration(ParameterDeclaration {
                modifiers,
                dot_dot_dot_token,
                name,
                question_token,
                type_node,
                initializer,
            }),
        )
    }

    /// Accepts `await` and `yield` as names here; rejecting them during speculation would cause
    /// many more follow-on errors than reporting them later.
    fn is_parameter_name_start(&self) -> bool {
        self.is_binding_identifier()
            || matches!(
                self.token,
                SyntaxKind::OpenBracketToken | SyntaxKind::OpenBraceToken
            )
    }

    fn parse_name_of_parameter(&mut self, modifiers: Option<&ModifierList>) -> NodeId {
        let name = self.parse_identifier_or_pattern_with_diagnostic(Some(
            diagnostics::PRIVATE_IDENTIFIERS_CANNOT_BE_USED_AS_PARAMETERS,
        ));
        let node = self.builder.node(name);
        // A modifier keyword that failed as a name, as in strict-mode `function f(static)`,
        // must still be consumed so the parameter list makes progress.
        if node.pos() == node.end() && modifiers.is_none() && self.token.is_modifier_kind() {
            self.next_token();
        }
        name
    }

    pub(super) fn parse_return_type(
        &mut self,
        return_token: SyntaxKind,
        is_type: bool,
    ) -> Option<NodeId> {
        self.should_parse_return_type(return_token, is_type)
            .then(|| {
                self.do_in_context(
                    NodeFlags::DISALLOW_CONDITIONAL_TYPES_CONTEXT,
                    false,
                    Self::parse_type_or_type_predicate,
                )
            })
    }

    fn should_parse_return_type(&mut self, return_token: SyntaxKind, is_type: bool) -> bool {
        if return_token == SyntaxKind::EqualsGreaterThanToken {
            self.parse_expected(return_token);
            return true;
        }
        if self.parse_optional(SyntaxKind::ColonToken) {
            return true;
        }
        if is_type && self.token == SyntaxKind::EqualsGreaterThanToken {
            // `=>` for `:` is an easy mistake in type contexts; parse the type anyway.
            self.parse_error_at_current_token(
                diagnostics::X_0_EXPECTED,
                &[token_to_string(SyntaxKind::ColonToken)],
            );
            self.next_token();
            return true;
        }
        false
    }

    pub(super) fn parse_type_or_type_predicate(&mut self) -> NodeId {
        if self.is_identifier() {
            let state = self.mark();
            let pos = self.node_pos();
            let parameter_name = self.parse_identifier();
            if self.token == SyntaxKind::IsKeyword && !self.has_preceding_line_break() {
                self.next_token();
                let type_node = Some(self.parse_type());
                return self.finish_node(
                    SyntaxKind::TypePredicate,
                    pos,
                    NodeData::TypePredicateNode(TypePredicateNode {
                        asserts_modifier: None,
                        parameter_name,
                        type_node,
                    }),
                );
            }
            self.rewind(state);
        }
        self.parse_type()
    }

    pub(super) fn parse_type_annotation(&mut self) -> Option<NodeId> {
        self.parse_optional(SyntaxKind::ColonToken)
            .then(|| self.parse_type())
    }

    pub(super) fn parse_initializer(&mut self) -> Option<NodeId> {
        self.parse_optional(SyntaxKind::EqualsToken)
            .then(|| self.parse_assignment_expression_or_higher())
    }

    pub(super) fn parse_function_block_or_semicolon(
        &mut self,
        flags: SignatureFlags,
        message: Option<Message>,
    ) -> Option<NodeId> {
        if self.token != SyntaxKind::OpenBraceToken {
            if flags.contains(SignatureFlags::TYPE) {
                self.parse_type_member_semicolon();
                return None;
            }
            if self.can_parse_semicolon() {
                self.parse_semicolon();
                return None;
            }
        }
        Some(self.parse_function_block(flags, message))
    }

    pub(super) fn parse_function_block(
        &mut self,
        flags: SignatureFlags,
        message: Option<Message>,
    ) -> NodeId {
        let saved_context_flags = self.context_flags;
        let saved_has_await_identifier = self.statement_has_await_identifier;
        self.set_context_flags(
            NodeFlags::YIELD_CONTEXT,
            flags.contains(SignatureFlags::YIELD),
        );
        self.set_context_flags(
            NodeFlags::AWAIT_CONTEXT,
            flags.contains(SignatureFlags::AWAIT),
        );
        // A function body never continues a [Decorator] context.
        self.set_context_flags(NodeFlags::DECORATOR_CONTEXT, false);
        let block = self.parse_block(
            flags.contains(SignatureFlags::IGNORE_MISSING_OPEN_BRACE),
            message,
        );
        self.context_flags = saved_context_flags;
        self.statement_has_await_identifier = saved_has_await_identifier;
        block
    }

    /// Accepts a comma or a (possibly inserted) semicolon after a type member.
    pub(super) fn parse_type_member_semicolon(&mut self) {
        if self.parse_optional(SyntaxKind::CommaToken) {
            return;
        }
        self.parse_semicolon();
    }
}
