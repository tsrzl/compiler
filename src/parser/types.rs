//! Type parsing.

use crate::ast::{
    ArrayTypeNode, ConditionalTypeNode, ConstructorTypeNode, FunctionTypeNode, ImportAttribute,
    ImportAttributes, ImportTypeNode, IndexedAccessTypeNode, InferTypeNode, IntersectionTypeNode,
    JSDocNonNullableType, JSDocNullableType, LiteralTypeNode, MappedTypeNode, NamedTupleMember,
    NodeData, NodeFlags, NodeId, NodeList, OptionalTypeNode, ParenthesizedTypeNode,
    PrefixUnaryExpression, QualifiedName, RestTypeNode, SyntaxKind, TemplateLiteralTypeNode,
    TemplateLiteralTypeSpan, TupleTypeNode, TypeLiteralNode, TypeOperatorNode,
    TypeParameterDeclaration, TypePredicateNode, TypeQueryNode, TypeReferenceNode, UnionTypeNode,
};
use crate::diagnostics::{self, Message};

use super::signatures::SignatureFlags;
use super::{ParseDiagnostic, Parser, ParsingContext, token_to_string};

impl Parser<'_> {
    pub(super) fn parse_type(&mut self) -> NodeId {
        let saved_context_flags = self.context_flags;
        self.set_context_flags(NodeFlags::TYPE_EXCLUDES_FLAGS, false);
        let type_node = if self.is_start_of_function_type_or_constructor_type() {
            self.parse_function_or_constructor_type()
        } else {
            let pos = self.node_pos();
            let check_type = self.parse_union_type_or_higher();
            if !self.in_context(NodeFlags::DISALLOW_CONDITIONAL_TYPES_CONTEXT)
                && !self.has_preceding_line_break()
                && self.parse_optional(SyntaxKind::ExtendsKeyword)
            {
                // The `extends` type of a conditional type cannot itself be conditional.
                let extends_type = self.do_in_context(
                    NodeFlags::DISALLOW_CONDITIONAL_TYPES_CONTEXT,
                    true,
                    Self::parse_type,
                );
                self.parse_expected(SyntaxKind::QuestionToken);
                let true_type = self.do_in_context(
                    NodeFlags::DISALLOW_CONDITIONAL_TYPES_CONTEXT,
                    false,
                    Self::parse_type,
                );
                self.parse_expected(SyntaxKind::ColonToken);
                let false_type = self.do_in_context(
                    NodeFlags::DISALLOW_CONDITIONAL_TYPES_CONTEXT,
                    false,
                    Self::parse_type,
                );
                self.finish_node(
                    SyntaxKind::ConditionalType,
                    pos,
                    NodeData::ConditionalTypeNode(ConditionalTypeNode {
                        check_type,
                        extends_type,
                        true_type,
                        false_type,
                    }),
                )
            } else {
                check_type
            }
        };
        self.context_flags = saved_context_flags;
        type_node
    }

    fn parse_union_type_or_higher(&mut self) -> NodeId {
        self.parse_union_or_intersection_type(
            SyntaxKind::BarToken,
            Self::parse_intersection_type_or_higher,
        )
    }

    fn parse_intersection_type_or_higher(&mut self) -> NodeId {
        self.parse_union_or_intersection_type(
            SyntaxKind::AmpersandToken,
            Self::parse_type_operator_or_higher,
        )
    }

    fn parse_union_or_intersection_type(
        &mut self,
        operator: SyntaxKind,
        parse_constituent_type: fn(&mut Self) -> NodeId,
    ) -> NodeId {
        let pos = self.node_pos();
        let is_union_type = operator == SyntaxKind::BarToken;
        let has_leading_operator = self.parse_optional(operator);
        let first = if has_leading_operator {
            self.parse_function_or_constructor_type_to_error(is_union_type, parse_constituent_type)
        } else {
            parse_constituent_type(self)
        };
        if self.token != operator && !has_leading_operator {
            return first;
        }
        let mut types = vec![first];
        while self.parse_optional(operator) {
            types.push(self.parse_function_or_constructor_type_to_error(
                is_union_type,
                parse_constituent_type,
            ));
        }
        let end = self.node_pos();
        let types = self.new_node_list(pos, end, types);
        let (kind, data) = if is_union_type {
            (
                SyntaxKind::UnionType,
                NodeData::UnionTypeNode(UnionTypeNode { types }),
            )
        } else {
            (
                SyntaxKind::IntersectionType,
                NodeData::IntersectionTypeNode(IntersectionTypeNode { types }),
            )
        };
        self.finish_node(kind, pos, data)
    }

    /// Parses a function or constructor type where it must be parenthesized, reporting it.
    fn parse_function_or_constructor_type_to_error(
        &mut self,
        is_in_union_type: bool,
        parse_constituent_type: fn(&mut Self) -> NodeId,
    ) -> NodeId {
        if !self.is_start_of_function_type_or_constructor_type() {
            return parse_constituent_type(self);
        }
        let type_node = self.parse_function_or_constructor_type();
        let node = self.builder.node(type_node);
        let message = match (node.kind() == SyntaxKind::FunctionType, is_in_union_type) {
            (true, true) => diagnostics::FUNCTION_TYPE_NOTATION_MUST_BE_PARENTHESIZED_WHEN_USED_IN_A_UNION_TYPE,
            (true, false) => diagnostics::FUNCTION_TYPE_NOTATION_MUST_BE_PARENTHESIZED_WHEN_USED_IN_AN_INTERSECTION_TYPE,
            (false, true) => diagnostics::CONSTRUCTOR_TYPE_NOTATION_MUST_BE_PARENTHESIZED_WHEN_USED_IN_A_UNION_TYPE,
            (false, false) => diagnostics::CONSTRUCTOR_TYPE_NOTATION_MUST_BE_PARENTHESIZED_WHEN_USED_IN_AN_INTERSECTION_TYPE,
        };
        let (pos, end) = (node.pos() as usize, node.end() as usize);
        self.parse_error_at(pos, end, message, &[]);
        type_node
    }

    fn parse_type_operator_or_higher(&mut self) -> NodeId {
        match self.token {
            SyntaxKind::KeyOfKeyword | SyntaxKind::UniqueKeyword | SyntaxKind::ReadonlyKeyword => {
                let pos = self.node_pos();
                let operator = self.token;
                self.parse_expected(operator);
                let type_node = self.parse_type_operator_or_higher();
                self.finish_node(
                    SyntaxKind::TypeOperator,
                    pos,
                    NodeData::TypeOperatorNode(TypeOperatorNode {
                        operator,
                        type_node,
                    }),
                )
            }
            SyntaxKind::InferKeyword => self.parse_infer_type(),
            _ => self.do_in_context(
                NodeFlags::DISALLOW_CONDITIONAL_TYPES_CONTEXT,
                false,
                Self::parse_postfix_type_or_higher,
            ),
        }
    }

    fn parse_infer_type(&mut self) -> NodeId {
        let pos = self.node_pos();
        self.parse_expected(SyntaxKind::InferKeyword);
        let parameter_pos = self.node_pos();
        let name = self.parse_identifier();
        let constraint = self.try_parse_constraint_of_infer_type();
        let type_parameter = self.finish_type_parameter(parameter_pos, name, constraint);
        self.finish_node(
            SyntaxKind::InferType,
            pos,
            NodeData::InferTypeNode(InferTypeNode { type_parameter }),
        )
    }

    fn finish_type_parameter(
        &mut self,
        pos: usize,
        name: NodeId,
        constraint: Option<NodeId>,
    ) -> NodeId {
        self.finish_node(
            SyntaxKind::TypeParameter,
            pos,
            NodeData::TypeParameterDeclaration(TypeParameterDeclaration {
                modifiers: None,
                name,
                constraint,
                expression: None,
                default_type: None,
            }),
        )
    }

    fn try_parse_constraint_of_infer_type(&mut self) -> Option<NodeId> {
        let state = self.mark();
        if self.parse_optional(SyntaxKind::ExtendsKeyword) {
            let constraint = self.do_in_context(
                NodeFlags::DISALLOW_CONDITIONAL_TYPES_CONTEXT,
                true,
                Self::parse_type,
            );
            if self.in_context(NodeFlags::DISALLOW_CONDITIONAL_TYPES_CONTEXT)
                || self.token != SyntaxKind::QuestionToken
            {
                return Some(constraint);
            }
        }
        self.rewind(state);
        None
    }

    fn parse_postfix_type_or_higher(&mut self) -> NodeId {
        let pos = self.node_pos();
        let mut type_node = self.parse_non_array_type();
        while !self.has_preceding_line_break() {
            match self.token {
                SyntaxKind::ExclamationToken => {
                    self.next_token();
                    type_node = self.finish_node(
                        SyntaxKind::JSDocNonNullableType,
                        pos,
                        NodeData::JSDocNonNullableType(JSDocNonNullableType { type_node }),
                    );
                }
                SyntaxKind::QuestionToken => {
                    // A following type makes this `?` part of a conditional type.
                    if self.look_ahead(|parser| {
                        parser.next_token();
                        parser.is_start_of_type(false)
                    }) {
                        return type_node;
                    }
                    self.next_token();
                    type_node = self.finish_node(
                        SyntaxKind::JSDocNullableType,
                        pos,
                        NodeData::JSDocNullableType(JSDocNullableType { type_node }),
                    );
                }
                SyntaxKind::OpenBracketToken => {
                    self.parse_expected(SyntaxKind::OpenBracketToken);
                    if self.is_start_of_type(false) {
                        let index_type = self.parse_type();
                        self.parse_expected(SyntaxKind::CloseBracketToken);
                        type_node = self.finish_node(
                            SyntaxKind::IndexedAccessType,
                            pos,
                            NodeData::IndexedAccessTypeNode(IndexedAccessTypeNode {
                                object_type: type_node,
                                index_type,
                            }),
                        );
                    } else {
                        self.parse_expected(SyntaxKind::CloseBracketToken);
                        type_node = self.finish_node(
                            SyntaxKind::ArrayType,
                            pos,
                            NodeData::ArrayTypeNode(ArrayTypeNode {
                                element_type: type_node,
                            }),
                        );
                    }
                }
                _ => return type_node,
            }
        }
        type_node
    }

    fn parse_non_array_type(&mut self) -> NodeId {
        match self.token {
            SyntaxKind::AnyKeyword
            | SyntaxKind::UnknownKeyword
            | SyntaxKind::StringKeyword
            | SyntaxKind::NumberKeyword
            | SyntaxKind::BigIntKeyword
            | SyntaxKind::SymbolKeyword
            | SyntaxKind::BooleanKeyword
            | SyntaxKind::UndefinedKeyword
            | SyntaxKind::NeverKeyword
            | SyntaxKind::ObjectKeyword => self.parse_keyword_type_or_dotted_type_reference(),
            SyntaxKind::AsteriskEqualsToken
            | SyntaxKind::AsteriskToken
            | SyntaxKind::QuestionQuestionToken
            | SyntaxKind::QuestionToken
            | SyntaxKind::ExclamationToken => self.parse_jsdoc_prefix_type(),
            SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::StringLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::NullKeyword => self.parse_literal_type_node(false),
            SyntaxKind::MinusToken => {
                if self.look_ahead(|parser| {
                    matches!(
                        parser.next_token(),
                        SyntaxKind::NumericLiteral | SyntaxKind::BigIntLiteral
                    )
                }) {
                    self.parse_literal_type_node(true)
                } else {
                    self.parse_type_reference()
                }
            }
            SyntaxKind::VoidKeyword => self.parse_keyword_type_node(),
            SyntaxKind::ThisKeyword => {
                let this_keyword = self.parse_this_type_node();
                if self.token == SyntaxKind::IsKeyword && !self.has_preceding_line_break() {
                    return self.parse_this_type_predicate(this_keyword);
                }
                this_keyword
            }
            SyntaxKind::TypeOfKeyword => {
                if self.look_ahead(|parser| parser.next_token() == SyntaxKind::ImportKeyword) {
                    self.parse_import_type()
                } else {
                    self.parse_type_query()
                }
            }
            SyntaxKind::OpenBraceToken => {
                if self.look_ahead(Self::next_is_start_of_mapped_type) {
                    self.parse_mapped_type()
                } else {
                    self.parse_type_literal()
                }
            }
            SyntaxKind::OpenBracketToken => self.parse_tuple_type(),
            SyntaxKind::OpenParenToken => self.parse_parenthesized_type(),
            SyntaxKind::ImportKeyword => self.parse_import_type(),
            SyntaxKind::AssertsKeyword => {
                if self.look_ahead(Self::next_token_is_identifier_or_keyword_on_same_line) {
                    self.parse_asserts_type_predicate()
                } else {
                    self.parse_type_reference()
                }
            }
            SyntaxKind::TemplateHead => self.parse_template_type(),
            _ => self.parse_type_reference(),
        }
    }

    /// Parses a keyword type, or a dotted type reference when the keyword is followed by `.`.
    fn parse_keyword_type_or_dotted_type_reference(&mut self) -> NodeId {
        let state = self.mark();
        let keyword_type = self.parse_keyword_type_node();
        if self.token != SyntaxKind::DotToken {
            return keyword_type;
        }
        self.rewind(state);
        self.parse_type_reference()
    }

    /// Parses the `JSDoc` prefix types `*`, `?T`, and `!T`, splitting `*=` and `??` first.
    fn parse_jsdoc_prefix_type(&mut self) -> NodeId {
        match self.token {
            SyntaxKind::AsteriskEqualsToken => {
                self.token = self.scanner.rescan_asterisk_equals_token();
            }
            SyntaxKind::QuestionQuestionToken => {
                self.token = self.scanner.rescan_question_token();
            }
            _ => {}
        }
        let pos = self.node_pos();
        let token = self.token;
        self.next_token();
        match token {
            SyntaxKind::AsteriskToken => {
                self.finish_node(SyntaxKind::JSDocAllType, pos, NodeData::JSDocAllType)
            }
            SyntaxKind::QuestionToken => {
                let type_node = self.parse_type_operator_or_higher();
                self.finish_node(
                    SyntaxKind::JSDocNullableType,
                    pos,
                    NodeData::JSDocNullableType(JSDocNullableType { type_node }),
                )
            }
            _ => {
                let type_node = self.parse_type_operator_or_higher();
                self.finish_node(
                    SyntaxKind::JSDocNonNullableType,
                    pos,
                    NodeData::JSDocNonNullableType(JSDocNonNullableType { type_node }),
                )
            }
        }
    }

    fn parse_keyword_type_node(&mut self) -> NodeId {
        let pos = self.node_pos();
        let kind = self.token;
        self.next_token();
        self.finish_node(kind, pos, NodeData::KeywordTypeNode)
    }

    fn parse_this_type_node(&mut self) -> NodeId {
        let pos = self.node_pos();
        self.next_token();
        self.finish_node(SyntaxKind::ThisType, pos, NodeData::ThisTypeNode)
    }

    fn parse_this_type_predicate(&mut self, parameter_name: NodeId) -> NodeId {
        self.next_token();
        let type_node = Some(self.parse_type());
        let pos = self.builder.node(parameter_name).pos() as usize;
        self.finish_node(
            SyntaxKind::TypePredicate,
            pos,
            NodeData::TypePredicateNode(TypePredicateNode {
                asserts_modifier: None,
                parameter_name,
                type_node,
            }),
        )
    }

    fn parse_literal_type_node(&mut self, negative: bool) -> NodeId {
        let pos = self.node_pos();
        if negative {
            self.next_token();
        }
        let mut literal = if matches!(
            self.token,
            SyntaxKind::TrueKeyword | SyntaxKind::FalseKeyword | SyntaxKind::NullKeyword
        ) {
            self.parse_keyword_expression()
        } else {
            self.parse_literal_expression()
        };
        if negative {
            literal = self.finish_node(
                SyntaxKind::PrefixUnaryExpression,
                pos,
                NodeData::PrefixUnaryExpression(PrefixUnaryExpression {
                    operator: SyntaxKind::MinusToken,
                    operand: literal,
                }),
            );
        }
        self.finish_node(
            SyntaxKind::LiteralType,
            pos,
            NodeData::LiteralTypeNode(LiteralTypeNode { literal }),
        )
    }

    pub(super) fn parse_type_reference(&mut self) -> NodeId {
        let pos = self.node_pos();
        let type_name = self.parse_entity_name_of_type_reference();
        let type_arguments = self.parse_type_arguments_of_type_reference();
        self.finish_node(
            SyntaxKind::TypeReference,
            pos,
            NodeData::TypeReferenceNode(TypeReferenceNode {
                type_name,
                type_arguments,
            }),
        )
    }

    fn parse_entity_name_of_type_reference(&mut self) -> NodeId {
        self.parse_entity_name(true, Some(diagnostics::TYPE_EXPECTED))
    }

    pub(super) fn parse_entity_name(
        &mut self,
        allow_reserved_words: bool,
        message: Option<Message>,
    ) -> NodeId {
        let pos = self.node_pos();
        let mut entity = if allow_reserved_words {
            self.parse_identifier_name_with_diagnostic(message)
        } else {
            self.parse_identifier_with_diagnostic(message, None)
        };
        while self.parse_optional(SyntaxKind::DotToken) {
            // A JSDoc-style generic (`A.<T>`) is reported by the checker from the gap left here.
            if self.token == SyntaxKind::LessThanToken {
                break;
            }
            let right = self.parse_right_side_of_dot(allow_reserved_words, false, true);
            entity = self.finish_node(
                SyntaxKind::QualifiedName,
                pos,
                NodeData::QualifiedName(QualifiedName {
                    left: entity,
                    right,
                }),
            );
        }
        entity
    }

    fn parse_type_arguments_of_type_reference(&mut self) -> Option<NodeList> {
        if !self.has_preceding_line_break()
            && self.rescan_less_than_token() == SyntaxKind::LessThanToken
        {
            return self.parse_type_arguments();
        }
        None
    }

    pub(super) fn parse_type_arguments(&mut self) -> Option<NodeList> {
        if self.token != SyntaxKind::LessThanToken {
            return None;
        }
        self.parse_bracketed_list(
            ParsingContext::TypeArguments,
            |parser| Some(parser.parse_type()),
            SyntaxKind::LessThanToken,
            SyntaxKind::GreaterThanToken,
        )
    }

    fn parse_import_type(&mut self) -> NodeId {
        self.source_flags |= NodeFlags::POSSIBLY_CONTAINS_DYNAMIC_IMPORT;
        let pos = self.node_pos();
        let is_type_of = self.parse_optional(SyntaxKind::TypeOfKeyword);
        self.parse_expected(SyntaxKind::ImportKeyword);
        self.parse_expected(SyntaxKind::OpenParenToken);
        let argument = self.parse_type();
        let mut attributes = None;
        if self.parse_optional(SyntaxKind::CommaToken) {
            let open_brace_position = self.scanner.token_start();
            self.parse_expected(SyntaxKind::OpenBraceToken);
            let current_token = self.token;
            if matches!(
                current_token,
                SyntaxKind::WithKeyword | SyntaxKind::AssertKeyword
            ) {
                if current_token == SyntaxKind::AssertKeyword {
                    self.parse_error_at_current_token(
                        diagnostics::IMPORT_ASSERTIONS_HAVE_BEEN_REPLACED_BY_IMPORT_ATTRIBUTES_USE_WITH_INSTEAD_OF_ASSERT,
                        &[],
                    );
                }
                self.next_token();
            } else {
                self.parse_error_at_current_token(
                    diagnostics::X_0_EXPECTED,
                    &[token_to_string(SyntaxKind::WithKeyword)],
                );
            }
            self.parse_expected(SyntaxKind::ColonToken);
            attributes = Some(self.parse_import_attributes(current_token, true));
            self.parse_optional(SyntaxKind::CommaToken);
            if !self.parse_expected(SyntaxKind::CloseBraceToken) {
                self.relate_missing_close_brace(open_brace_position);
            }
        }
        self.parse_expected(SyntaxKind::CloseParenToken);
        let qualifier = self
            .parse_optional(SyntaxKind::DotToken)
            .then(|| self.parse_entity_name_of_type_reference());
        let type_arguments = self.parse_type_arguments_of_type_reference();
        self.finish_node(
            SyntaxKind::ImportType,
            pos,
            NodeData::ImportTypeNode(ImportTypeNode {
                is_type_of,
                argument,
                attributes,
                qualifier,
                type_arguments,
            }),
        )
    }

    /// Relates a just-reported missing `}` to the `{` that it should match.
    fn relate_missing_close_brace(&mut self, open_brace_position: usize) {
        let Some(last) = self.diagnostics.last_mut() else {
            return;
        };
        if last.message.code() == diagnostics::X_0_EXPECTED.code() {
            last.related.push(ParseDiagnostic::new(
                diagnostics::THE_PARSER_EXPECTED_TO_FIND_A_1_TO_MATCH_THE_0_TOKEN_HERE,
                open_brace_position,
                open_brace_position,
                &["{", "}"],
            ));
        }
    }

    fn parse_import_attribute(&mut self) -> NodeId {
        let pos = self.node_pos();
        let name = if self.token.is_identifier_or_keyword() {
            Some(self.parse_identifier_name())
        } else if self.token == SyntaxKind::StringLiteral {
            Some(self.parse_literal_expression())
        } else {
            None
        };
        if name.is_some() {
            self.parse_expected(SyntaxKind::ColonToken);
        } else {
            self.parse_error_at_current_token(
                diagnostics::IDENTIFIER_OR_STRING_LITERAL_EXPECTED,
                &[],
            );
        }
        let value = self.parse_assignment_expression_or_higher();
        self.finish_node(
            SyntaxKind::ImportAttribute,
            pos,
            NodeData::ImportAttribute(ImportAttribute { name, value }),
        )
    }

    pub(super) fn parse_import_attributes(
        &mut self,
        token: SyntaxKind,
        skip_keyword: bool,
    ) -> NodeId {
        let pos = self.node_pos();
        if !skip_keyword {
            self.parse_expected(token);
        }
        let open_brace_position = self.scanner.token_start();
        let (attributes, multi_line) = if self.parse_expected(SyntaxKind::OpenBraceToken) {
            let multi_line = self.has_preceding_line_break();
            let attributes = self
                .parse_delimited_list(ParsingContext::ImportAttributes, |parser| {
                    Some(parser.parse_import_attribute())
                })
                .expect("import attributes always parse");
            if !self.parse_expected(SyntaxKind::CloseBraceToken) {
                self.relate_missing_close_brace(open_brace_position);
            }
            (attributes, multi_line)
        } else {
            let pos = self.node_pos();
            (self.new_node_list(pos, pos, Vec::new()), false)
        };
        self.finish_node(
            SyntaxKind::ImportAttributes,
            pos,
            NodeData::ImportAttributes(ImportAttributes {
                token,
                attributes,
                multi_line,
            }),
        )
    }

    fn parse_type_query(&mut self) -> NodeId {
        let pos = self.node_pos();
        self.parse_expected(SyntaxKind::TypeOfKeyword);
        let expr_name = self.parse_entity_name(true, None);
        // ASI keeps the next line's `<` from becoming instantiation type arguments.
        let type_arguments = if self.has_preceding_line_break() {
            None
        } else {
            self.parse_type_arguments()
        };
        self.finish_node(
            SyntaxKind::TypeQuery,
            pos,
            NodeData::TypeQueryNode(TypeQueryNode {
                expr_name,
                type_arguments,
            }),
        )
    }

    fn next_is_start_of_mapped_type(&mut self) -> bool {
        self.next_token();
        if matches!(self.token, SyntaxKind::PlusToken | SyntaxKind::MinusToken) {
            return self.next_token() == SyntaxKind::ReadonlyKeyword;
        }
        if self.token == SyntaxKind::ReadonlyKeyword {
            self.next_token();
        }
        self.token == SyntaxKind::OpenBracketToken
            && {
                self.next_token();
                self.is_identifier()
            }
            && self.next_token() == SyntaxKind::InKeyword
    }

    fn parse_mapped_type(&mut self) -> NodeId {
        let pos = self.node_pos();
        self.parse_expected(SyntaxKind::OpenBraceToken);
        let mut readonly_token = None;
        if matches!(
            self.token,
            SyntaxKind::ReadonlyKeyword | SyntaxKind::PlusToken | SyntaxKind::MinusToken
        ) {
            let token = self.parse_token_node();
            if self.builder.node(token).kind() != SyntaxKind::ReadonlyKeyword {
                self.parse_expected(SyntaxKind::ReadonlyKeyword);
            }
            readonly_token = Some(token);
        }
        self.parse_expected(SyntaxKind::OpenBracketToken);
        let parameter_pos = self.node_pos();
        let name = self.parse_identifier_name();
        self.parse_expected(SyntaxKind::InKeyword);
        let constraint = self.parse_type();
        let type_parameter = self.finish_type_parameter(parameter_pos, name, Some(constraint));
        let name_type = self
            .parse_optional(SyntaxKind::AsKeyword)
            .then(|| self.parse_type());
        self.parse_expected(SyntaxKind::CloseBracketToken);
        let mut question_token = None;
        if matches!(
            self.token,
            SyntaxKind::QuestionToken | SyntaxKind::PlusToken | SyntaxKind::MinusToken
        ) {
            let token = self.parse_token_node();
            if self.builder.node(token).kind() != SyntaxKind::QuestionToken {
                self.parse_expected(SyntaxKind::QuestionToken);
            }
            question_token = Some(token);
        }
        let type_node = self.parse_type_annotation();
        self.parse_semicolon();
        let members = Some(self.parse_list(ParsingContext::TypeMembers, Self::parse_type_member));
        self.parse_expected(SyntaxKind::CloseBraceToken);
        self.finish_node(
            SyntaxKind::MappedType,
            pos,
            NodeData::MappedTypeNode(MappedTypeNode {
                readonly_token,
                type_parameter,
                name_type,
                question_token,
                type_node,
                members,
            }),
        )
    }

    fn parse_type_literal(&mut self) -> NodeId {
        let pos = self.node_pos();
        let members = self.parse_object_type_members();
        self.finish_node(
            SyntaxKind::TypeLiteral,
            pos,
            NodeData::TypeLiteralNode(TypeLiteralNode { members }),
        )
    }

    pub(super) fn parse_object_type_members(&mut self) -> NodeList {
        if self.parse_expected(SyntaxKind::OpenBraceToken) {
            let members = self.parse_list(ParsingContext::TypeMembers, Self::parse_type_member);
            self.parse_expected(SyntaxKind::CloseBraceToken);
            return members;
        }
        let pos = self.node_pos();
        self.builder.add_missing_list(super::to_u32(pos))
    }

    fn parse_tuple_type(&mut self) -> NodeId {
        let pos = self.node_pos();
        let elements = self
            .parse_bracketed_list(
                ParsingContext::TupleElementTypes,
                |parser| Some(parser.parse_tuple_element_name_or_tuple_element_type()),
                SyntaxKind::OpenBracketToken,
                SyntaxKind::CloseBracketToken,
            )
            .expect("tuple elements always parse");
        self.finish_node(
            SyntaxKind::TupleType,
            pos,
            NodeData::TupleTypeNode(TupleTypeNode { elements }),
        )
    }

    fn parse_tuple_element_name_or_tuple_element_type(&mut self) -> NodeId {
        if !self.look_ahead(Self::scan_start_of_named_tuple_element) {
            return self.parse_tuple_element_type();
        }
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        let dot_dot_dot_token = self.parse_optional_token(SyntaxKind::DotDotDotToken);
        let name = self.parse_identifier_name();
        let question_token = self.parse_optional_token(SyntaxKind::QuestionToken);
        self.parse_expected(SyntaxKind::ColonToken);
        let type_node = self.parse_tuple_element_type();
        let result = self.finish_node(
            SyntaxKind::NamedTupleMember,
            pos,
            NodeData::NamedTupleMember(NamedTupleMember {
                dot_dot_dot_token,
                name,
                question_token,
                type_node,
            }),
        );
        self.with_jsdoc(result, jsdoc);
        result
    }

    fn scan_start_of_named_tuple_element(&mut self) -> bool {
        if self.token == SyntaxKind::DotDotDotToken {
            return self.next_token().is_identifier_or_keyword()
                && self.next_token_is_colon_or_question_colon();
        }
        self.token.is_identifier_or_keyword() && self.next_token_is_colon_or_question_colon()
    }

    fn next_token_is_colon_or_question_colon(&mut self) -> bool {
        self.next_token() == SyntaxKind::ColonToken
            || (self.token == SyntaxKind::QuestionToken
                && self.next_token() == SyntaxKind::ColonToken)
    }

    fn parse_tuple_element_type(&mut self) -> NodeId {
        let pos = self.node_pos();
        if self.parse_optional(SyntaxKind::DotDotDotToken) {
            let type_node = self.parse_type();
            return self.finish_node(
                SyntaxKind::RestType,
                pos,
                NodeData::RestTypeNode(RestTypeNode { type_node }),
            );
        }
        let type_node = self.parse_type();
        // A postfix `T?` in a tuple is an optional element, not a nullable type.
        let node = self.builder.node(type_node);
        if let Some(nullable) = node.data().as_jsdoc_nullable_type() {
            let inner = nullable.type_node;
            if node.pos() == self.builder.node(inner).pos() {
                let (pos, end, flags) = (node.pos(), node.end(), node.flags());
                return self.builder.add_node(
                    SyntaxKind::OptionalType,
                    pos,
                    end,
                    flags,
                    NodeData::OptionalTypeNode(OptionalTypeNode { type_node: inner }),
                );
            }
        }
        type_node
    }

    fn parse_parenthesized_type(&mut self) -> NodeId {
        let pos = self.node_pos();
        self.parse_expected(SyntaxKind::OpenParenToken);
        let type_node = self.parse_type();
        self.parse_expected(SyntaxKind::CloseParenToken);
        self.finish_node(
            SyntaxKind::ParenthesizedType,
            pos,
            NodeData::ParenthesizedTypeNode(ParenthesizedTypeNode { type_node }),
        )
    }

    fn parse_asserts_type_predicate(&mut self) -> NodeId {
        let pos = self.node_pos();
        let asserts_modifier = Some(self.parse_expected_token(SyntaxKind::AssertsKeyword));
        let parameter_name = if self.token == SyntaxKind::ThisKeyword {
            self.parse_this_type_node()
        } else {
            self.parse_identifier()
        };
        let type_node = self
            .parse_optional(SyntaxKind::IsKeyword)
            .then(|| self.parse_type());
        self.finish_node(
            SyntaxKind::TypePredicate,
            pos,
            NodeData::TypePredicateNode(TypePredicateNode {
                asserts_modifier,
                parameter_name,
                type_node,
            }),
        )
    }

    fn parse_template_type(&mut self) -> NodeId {
        let pos = self.node_pos();
        let head = self.parse_template_head(false);
        let spans_pos = self.node_pos();
        let mut spans = Vec::new();
        loop {
            let span_start = self.node_pos();
            let type_node = self.parse_type();
            let literal = self.parse_literal_of_template_span(false);
            spans.push(self.finish_node(
                SyntaxKind::TemplateLiteralTypeSpan,
                span_start,
                NodeData::TemplateLiteralTypeSpan(TemplateLiteralTypeSpan { type_node, literal }),
            ));
            if self.builder.node(literal).kind() != SyntaxKind::TemplateMiddle {
                break;
            }
        }
        let end = self.node_pos();
        let template_spans = self.new_node_list(spans_pos, end, spans);
        self.finish_node(
            SyntaxKind::TemplateLiteralType,
            pos,
            NodeData::TemplateLiteralTypeNode(TemplateLiteralTypeNode {
                head,
                template_spans,
            }),
        )
    }

    fn is_start_of_function_type_or_constructor_type(&mut self) -> bool {
        match self.token {
            SyntaxKind::LessThanToken | SyntaxKind::NewKeyword => true,
            SyntaxKind::OpenParenToken => {
                self.look_ahead(Self::next_is_unambiguously_start_of_function_type)
            }
            SyntaxKind::AbstractKeyword => {
                self.look_ahead(|parser| parser.next_token() == SyntaxKind::NewKeyword)
            }
            _ => false,
        }
    }

    fn parse_function_or_constructor_type(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        let modifiers = self.parse_modifiers_for_constructor_type();
        let is_constructor_type = self.parse_optional(SyntaxKind::NewKeyword);
        assert!(
            modifiers.is_none() || is_constructor_type,
            "a function type cannot have modifiers"
        );
        let type_parameters = self.parse_type_parameters();
        let parameters = self.parse_parameters(SignatureFlags::TYPE);
        let type_node = self.parse_return_type(SyntaxKind::EqualsGreaterThanToken, false);
        let result = if is_constructor_type {
            self.finish_node(
                SyntaxKind::ConstructorType,
                pos,
                NodeData::ConstructorTypeNode(ConstructorTypeNode {
                    modifiers,
                    type_parameters,
                    parameters,
                    type_node,
                }),
            )
        } else {
            self.finish_node(
                SyntaxKind::FunctionType,
                pos,
                NodeData::FunctionTypeNode(FunctionTypeNode {
                    type_parameters,
                    parameters,
                    type_node,
                }),
            )
        };
        self.with_jsdoc(result, jsdoc);
        result
    }

    fn parse_modifiers_for_constructor_type(&mut self) -> Option<crate::ast::ModifierList> {
        if self.token != SyntaxKind::AbstractKeyword {
            return None;
        }
        let pos = self.node_pos();
        self.next_token();
        let modifier = self.finish_node(SyntaxKind::AbstractKeyword, pos, NodeData::Token);
        let end = self.builder.node(modifier).end() as usize;
        Some(self.new_modifier_list(pos, end, vec![modifier]))
    }

    fn next_is_unambiguously_start_of_function_type(&mut self) -> bool {
        self.next_token();
        if matches!(
            self.token,
            SyntaxKind::CloseParenToken | SyntaxKind::DotDotDotToken
        ) {
            return true;
        }
        if self.skip_parameter_start() {
            if matches!(
                self.token,
                SyntaxKind::ColonToken
                    | SyntaxKind::CommaToken
                    | SyntaxKind::QuestionToken
                    | SyntaxKind::EqualsToken
            ) {
                return true;
            }
            if self.token == SyntaxKind::CloseParenToken
                && self.next_token() == SyntaxKind::EqualsGreaterThanToken
            {
                return true;
            }
        }
        false
    }

    fn skip_parameter_start(&mut self) -> bool {
        if self.token.is_modifier_kind() {
            self.parse_modifiers();
        }
        self.parse_optional(SyntaxKind::DotDotDotToken);
        if self.is_identifier() || self.token == SyntaxKind::ThisKeyword {
            self.next_token();
            return true;
        }
        if matches!(
            self.token,
            SyntaxKind::OpenBracketToken | SyntaxKind::OpenBraceToken
        ) {
            // Accept a binding pattern only if it parses without errors.
            let error_count = self.diagnostics.len();
            self.parse_identifier_or_pattern();
            return error_count == self.diagnostics.len();
        }
        false
    }
}
