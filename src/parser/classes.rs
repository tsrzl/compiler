//! Classes: declarations, expressions, heritage clauses, and class elements.

use crate::ast::{
    ClassDeclaration, ClassExpression, ClassStaticBlockDeclaration, ConstructorDeclaration,
    ExpressionWithTypeArguments, HeritageClause, ModifierFlags, ModifierList, NodeData, NodeFlags,
    NodeId, NodeList, PropertyDeclaration, SyntaxKind,
};
use crate::diagnostics;

use super::modifiers::ModifierOptions;
use super::signatures::SignatureFlags;
use super::{JsdocScannerInfo, Parser, ParsingContext, token_to_string};

impl Parser<'_> {
    pub(super) fn parse_class_expression(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        self.parse_class_declaration_or_expression(pos, jsdoc, None, SyntaxKind::ClassExpression)
    }

    /// Parses decorators before a class expression; anything else is reported as missing.
    pub(super) fn parse_decorated_expression(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        let modifiers = self.parse_modifiers_with(ModifierOptions::DECORATED);
        if self.token == SyntaxKind::ClassKeyword {
            return self.parse_class_declaration_or_expression(
                pos,
                jsdoc,
                modifiers,
                SyntaxKind::ClassExpression,
            );
        }
        let node_pos = self.node_pos();
        self.parse_error_at(node_pos, node_pos, diagnostics::EXPRESSION_EXPECTED, &[]);
        self.finish_node(
            SyntaxKind::MissingDeclaration,
            pos,
            NodeData::MissingDeclaration(crate::ast::MissingDeclaration { modifiers }),
        )
    }

    pub(super) fn parse_class_declaration_or_expression(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
        kind: SyntaxKind,
    ) -> NodeId {
        let saved_context_flags = self.context_flags;
        let saved_has_await_identifier = self.statement_has_await_identifier;
        self.parse_expected(SyntaxKind::ClassKeyword);
        // The name is not parsed in an await context; the checker reports that grammar error.
        let name = self.parse_name_of_class_declaration_or_expression();
        let type_parameters = self.parse_type_parameters();
        let modifier_flags = modifiers.map_or(ModifierFlags::NONE, |modifiers| modifiers.flags());
        if modifier_flags.intersects(ModifierFlags::EXPORT) {
            self.set_context_flags(NodeFlags::AWAIT_CONTEXT, true);
        }
        let heritage_clauses = self.parse_heritage_clauses();
        let members = if self.parse_expected(SyntaxKind::OpenBraceToken) {
            let members = self.parse_list(ParsingContext::ClassMembers, Self::parse_class_element);
            self.parse_expected(SyntaxKind::CloseBraceToken);
            members
        } else {
            let pos = self.node_pos();
            self.builder.add_missing_list(super::to_u32(pos))
        };
        self.context_flags = saved_context_flags;
        if modifier_flags.intersects(ModifierFlags::AMBIENT) {
            self.statement_has_await_identifier = saved_has_await_identifier;
        }
        let data = if kind == SyntaxKind::ClassDeclaration {
            NodeData::ClassDeclaration(ClassDeclaration {
                modifiers,
                name,
                type_parameters,
                heritage_clauses,
                members,
            })
        } else {
            NodeData::ClassExpression(ClassExpression {
                modifiers,
                name,
                type_parameters,
                heritage_clauses,
                members,
            })
        };
        let result = self.finish_node(kind, pos, data);
        self.with_jsdoc(result, jsdoc);
        result
    }

    /// `class implements` may be an unnamed class expression or a class named `implements`.
    fn parse_name_of_class_declaration_or_expression(&mut self) -> Option<NodeId> {
        if !self.is_binding_identifier() || self.is_implements_clause() {
            return None;
        }
        let saved = self.statement_has_await_identifier;
        let is_identifier = self.is_binding_identifier();
        let id = self.create_identifier(is_identifier);
        self.statement_has_await_identifier = saved;
        Some(id)
    }

    fn is_implements_clause(&mut self) -> bool {
        self.token == SyntaxKind::ImplementsKeyword
            && self.look_ahead(|parser| parser.next_token().is_identifier_or_keyword())
    }

    pub(super) fn parse_heritage_clauses(&mut self) -> Option<NodeList> {
        self.is_heritage_clause()
            .then(|| self.parse_list(ParsingContext::HeritageClauses, Self::parse_heritage_clause))
    }

    fn parse_heritage_clause(&mut self) -> NodeId {
        let pos = self.node_pos();
        let token = self.token;
        self.next_token();
        let types = self
            .parse_delimited_list(ParsingContext::HeritageClauseElement, |parser| {
                Some(parser.parse_expression_with_type_arguments())
            })
            .expect("heritage clause elements always parse");
        self.finish_node(
            SyntaxKind::HeritageClause,
            pos,
            NodeData::HeritageClause(HeritageClause { token, types }),
        )
    }

    fn parse_expression_with_type_arguments(&mut self) -> NodeId {
        let pos = self.node_pos();
        let expression = self.parse_left_hand_side_expression_or_higher();
        if self.builder.node(expression).kind() == SyntaxKind::ExpressionWithTypeArguments {
            return expression;
        }
        let type_arguments = self.parse_type_arguments();
        self.finish_node(
            SyntaxKind::ExpressionWithTypeArguments,
            pos,
            NodeData::ExpressionWithTypeArguments(ExpressionWithTypeArguments {
                expression,
                type_arguments,
            }),
        )
    }

    fn parse_class_element(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        if self.token == SyntaxKind::SemicolonToken {
            self.next_token();
            let result = self.finish_node(
                SyntaxKind::SemicolonClassElement,
                pos,
                NodeData::SemicolonClassElement,
            );
            self.with_jsdoc(result, jsdoc);
            return result;
        }
        let modifiers = self.parse_modifiers_with(ModifierOptions {
            allow_decorators: true,
            permit_const_as_modifier: true,
            stop_on_start_of_class_static_block: true,
        });
        if self.token == SyntaxKind::StaticKeyword
            && self.look_ahead(|parser| parser.next_token() == SyntaxKind::OpenBraceToken)
        {
            return self.parse_class_static_block_declaration(pos, jsdoc, modifiers);
        }
        for (keyword, kind) in [
            (SyntaxKind::GetKeyword, SyntaxKind::GetAccessor),
            (SyntaxKind::SetKeyword, SyntaxKind::SetAccessor),
        ] {
            if self.parse_contextual_modifier(keyword) {
                return self.parse_accessor_declaration(
                    pos,
                    jsdoc,
                    modifiers,
                    kind,
                    SignatureFlags::NONE,
                );
            }
        }
        if matches!(
            self.token,
            SyntaxKind::ConstructorKeyword | SyntaxKind::StringLiteral
        ) && let Some(constructor) =
            self.try_parse_constructor_declaration(pos, jsdoc, modifiers)
        {
            return constructor;
        }
        if self.is_index_signature() {
            return self.parse_index_signature_declaration(pos, jsdoc, modifiers);
        }
        // Checked after index signatures because `[` can start either an index signature or a
        // computed property name.
        if self.token.is_identifier_or_keyword()
            || matches!(
                self.token,
                SyntaxKind::StringLiteral
                    | SyntaxKind::NumericLiteral
                    | SyntaxKind::BigIntLiteral
                    | SyntaxKind::AsteriskToken
                    | SyntaxKind::OpenBracketToken
            )
        {
            let is_ambient = modifiers
                .is_some_and(|modifiers| modifiers.flags().intersects(ModifierFlags::AMBIENT));
            if !is_ambient {
                return self.parse_property_or_method_declaration(pos, jsdoc, modifiers);
            }
            if let Some(modifiers) = &modifiers {
                self.mark_modifiers_ambient(modifiers);
            }
            return self.do_in_context(NodeFlags::AMBIENT, true, |parser| {
                parser.parse_property_or_method_declaration(pos, jsdoc, modifiers)
            });
        }
        // Modifiers without a member: recover with a property that has a missing name.
        let modifiers = modifiers.expect("class member parsing starts only at a member");
        let node_pos = self.node_pos();
        self.parse_error_at(node_pos, node_pos, diagnostics::DECLARATION_EXPECTED, &[]);
        let name = self.create_missing_identifier();
        self.parse_property_declaration(pos, jsdoc, Some(modifiers), name, None)
    }

    fn parse_class_static_block_declaration(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
    ) -> NodeId {
        self.parse_expected_token(SyntaxKind::StaticKeyword);
        let saved_context_flags = self.context_flags;
        self.set_context_flags(NodeFlags::YIELD_CONTEXT, false);
        self.set_context_flags(NodeFlags::AWAIT_CONTEXT, true);
        let body = self.parse_block(false, None);
        self.context_flags = saved_context_flags;
        let result = self.finish_node(
            SyntaxKind::ClassStaticBlockDeclaration,
            pos,
            NodeData::ClassStaticBlockDeclaration(ClassStaticBlockDeclaration { modifiers, body }),
        );
        self.with_jsdoc(result, jsdoc);
        result
    }

    fn try_parse_constructor_declaration(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
    ) -> Option<NodeId> {
        let state = self.mark();
        let is_constructor = self.token == SyntaxKind::ConstructorKeyword
            || (self.token == SyntaxKind::StringLiteral
                && self.scanner.token_value() == "constructor"
                && self.look_ahead(|parser| parser.next_token() == SyntaxKind::OpenParenToken));
        if !is_constructor {
            self.rewind(state);
            return None;
        }
        self.next_token();
        let type_parameters = self.parse_type_parameters();
        let parameters = self.parse_parameters(SignatureFlags::NONE);
        let type_node = self.parse_return_type(SyntaxKind::ColonToken, false);
        let body = self.parse_function_block_or_semicolon(
            SignatureFlags::NONE,
            Some(diagnostics::X_OR_EXPECTED),
        );
        let result = self.finish_node(
            SyntaxKind::Constructor,
            pos,
            NodeData::ConstructorDeclaration(ConstructorDeclaration {
                modifiers,
                type_parameters,
                parameters,
                type_node,
                full_signature: None,
                body,
            }),
        );
        self.with_jsdoc(result, jsdoc);
        Some(result)
    }

    fn parse_property_or_method_declaration(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
    ) -> NodeId {
        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);
        let name = self.parse_property_name();
        // `?` is not grammatical on methods, but is parsed and reported by the grammar checker.
        let question_token = self.parse_optional_token(SyntaxKind::QuestionToken);
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
                question_token,
                Some(diagnostics::X_OR_EXPECTED),
            );
        }
        self.parse_property_declaration(pos, jsdoc, modifiers, name, question_token)
    }

    fn parse_property_declaration(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
        name: NodeId,
        question_token: Option<NodeId>,
    ) -> NodeId {
        let postfix_token = question_token.or_else(|| {
            (!self.has_preceding_line_break())
                .then(|| self.parse_optional_token(SyntaxKind::ExclamationToken))
                .flatten()
        });
        let type_node = self.parse_type_annotation();
        let initializer = self.do_in_context(
            NodeFlags::YIELD_CONTEXT | NodeFlags::AWAIT_CONTEXT | NodeFlags::DISALLOW_IN_CONTEXT,
            false,
            Self::parse_initializer,
        );
        self.parse_semicolon_after_property_name(name, type_node, initializer);
        let result = self.finish_node(
            SyntaxKind::PropertyDeclaration,
            pos,
            NodeData::PropertyDeclaration(PropertyDeclaration {
                modifiers,
                name,
                postfix_token,
                type_node,
                initializer,
            }),
        );
        self.with_jsdoc(result, jsdoc);
        result
    }

    fn parse_semicolon_after_property_name(
        &mut self,
        name: NodeId,
        type_node: Option<NodeId>,
        initializer: Option<NodeId>,
    ) {
        if self.token == SyntaxKind::AtToken && !self.has_preceding_line_break() {
            self.parse_error_at_current_token(
                diagnostics::DECORATORS_MUST_PRECEDE_THE_NAME_AND_ALL_KEYWORDS_OF_PROPERTY_DECLARATIONS,
                &[],
            );
            return;
        }
        if self.token == SyntaxKind::OpenParenToken {
            self.parse_error_at_current_token(
                diagnostics::CANNOT_START_A_FUNCTION_CALL_IN_A_TYPE_ANNOTATION,
                &[],
            );
            self.next_token();
            return;
        }
        let semicolon = token_to_string(SyntaxKind::SemicolonToken);
        if type_node.is_some() && !self.can_parse_semicolon() {
            if initializer.is_some() {
                self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &[semicolon]);
            } else {
                self.parse_error_at_current_token(
                    diagnostics::EXPECTED_FOR_PROPERTY_INITIALIZER,
                    &[],
                );
            }
            return;
        }
        if self.try_parse_semicolon() {
            return;
        }
        if initializer.is_some() {
            self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &[semicolon]);
            return;
        }
        self.parse_error_for_missing_semicolon_after(name);
    }
}
