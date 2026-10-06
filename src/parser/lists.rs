//! Parsing contexts and list parsing with TypeScript-Go's error recovery.

use crate::ast::{NodeId, NodeList, SyntaxKind};
use crate::diagnostics;

use super::{Parser, token_to_string};

/// A syntactic list being parsed, used to decide where a list ends during error recovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ParsingContext {
    /// Elements in a source file.
    SourceElements,
    /// Statements in a block.
    BlockStatements,
    /// Clauses in a switch statement.
    SwitchClauses,
    /// Statements in a switch clause.
    SwitchClauseStatements,
    /// Members in an interface or type literal.
    TypeMembers,
    /// Members in a class.
    ClassMembers,
    /// Members in an enum.
    EnumMembers,
    /// Elements in a heritage clause.
    HeritageClauseElement,
    /// Declarations in a variable statement.
    VariableDeclarations,
    /// Elements in an object binding pattern.
    ObjectBindingElements,
    /// Elements in an array binding pattern.
    ArrayBindingElements,
    /// Arguments in a call.
    ArgumentExpressions,
    /// Members in an object literal.
    ObjectLiteralMembers,
    /// Attributes in a JSX element.
    JsxAttributes,
    /// Children between JSX tags.
    JsxChildren,
    /// Members in an array literal.
    ArrayLiteralMembers,
    /// Parameters in a parameter list.
    Parameters,
    /// Parameters in a `JSDoc` function type.
    JsdocParameters,
    /// Property names in a rest type list.
    RestProperties,
    /// Type parameters.
    TypeParameters,
    /// Type arguments.
    TypeArguments,
    /// Element types in a tuple type.
    TupleElementTypes,
    /// Heritage clauses of a class or interface.
    HeritageClauses,
    /// Specifiers in a named import or export clause.
    ImportOrExportSpecifiers,
    /// Import attributes.
    ImportAttributes,
    /// A `JSDoc` comment.
    JsdocComment,
}

impl ParsingContext {
    const ALL: [Self; 26] = [
        Self::SourceElements,
        Self::BlockStatements,
        Self::SwitchClauses,
        Self::SwitchClauseStatements,
        Self::TypeMembers,
        Self::ClassMembers,
        Self::EnumMembers,
        Self::HeritageClauseElement,
        Self::VariableDeclarations,
        Self::ObjectBindingElements,
        Self::ArrayBindingElements,
        Self::ArgumentExpressions,
        Self::ObjectLiteralMembers,
        Self::JsxAttributes,
        Self::JsxChildren,
        Self::ArrayLiteralMembers,
        Self::Parameters,
        Self::JsdocParameters,
        Self::RestProperties,
        Self::TypeParameters,
        Self::TypeArguments,
        Self::TupleElementTypes,
        Self::HeritageClauses,
        Self::ImportOrExportSpecifiers,
        Self::ImportAttributes,
        Self::JsdocComment,
    ];

    const fn bit(self) -> u32 {
        1 << self as u32
    }
}

impl Parser<'_> {
    /// Parses list elements until the list terminates, passing each element's index.
    pub(super) fn parse_list_with_index(
        &mut self,
        kind: ParsingContext,
        mut parse_element: impl FnMut(&mut Self, usize) -> NodeId,
    ) -> Vec<NodeId> {
        let saved_contexts = self.parsing_contexts;
        self.parsing_contexts |= kind.bit();
        let mut list = Vec::new();
        while !self.is_list_terminator(kind) {
            if self.is_list_element(kind, false) {
                let index = list.len();
                list.push(parse_element(self, index));
                continue;
            }
            if self.abort_parsing_list_or_move_to_next_token(kind) {
                break;
            }
        }
        self.parsing_contexts = saved_contexts;
        list
    }

    /// Parses a list of elements without separators.
    pub(super) fn parse_list(
        &mut self,
        kind: ParsingContext,
        mut parse_element: impl FnMut(&mut Self) -> NodeId,
    ) -> NodeList {
        let pos = self.node_pos();
        let nodes = self.parse_list_with_index(kind, |parser, _| parse_element(parser));
        let end = self.node_pos();
        self.new_node_list(pos, end, nodes)
    }

    /// Parses a comma-separated list. Returns `None` when an element fails to parse.
    pub(super) fn parse_delimited_list(
        &mut self,
        kind: ParsingContext,
        mut parse_element: impl FnMut(&mut Self) -> Option<NodeId>,
    ) -> Option<NodeList> {
        let pos = self.node_pos();
        let saved_contexts = self.parsing_contexts;
        self.parsing_contexts |= kind.bit();
        let mut list = Vec::new();
        loop {
            if self.is_list_element(kind, false) {
                let start_pos = self.node_pos();
                let Some(element) = parse_element(self) else {
                    self.parsing_contexts = saved_contexts;
                    return None;
                };
                list.push(element);
                if self.parse_optional(SyntaxKind::CommaToken) {
                    continue;
                }
                if self.is_list_terminator(kind) {
                    break;
                }
                if self.token != SyntaxKind::CommaToken && kind == ParsingContext::EnumMembers {
                    self.parse_error_at_current_token(
                        diagnostics::AN_ENUM_MEMBER_NAME_MUST_BE_FOLLOWED_BY_A_OR,
                        &[],
                    );
                } else {
                    self.parse_expected(SyntaxKind::CommaToken);
                }
                if matches!(
                    kind,
                    ParsingContext::ObjectLiteralMembers | ParsingContext::ImportAttributes
                ) && self.token == SyntaxKind::SemicolonToken
                    && !self.has_preceding_line_break()
                {
                    self.next_token();
                }
                if start_pos == self.node_pos() {
                    // Nothing was consumed; advance to avoid looping forever.
                    self.next_token();
                }
                continue;
            }
            if self.is_list_terminator(kind) {
                break;
            }
            if self.abort_parsing_list_or_move_to_next_token(kind) {
                break;
            }
        }
        self.parsing_contexts = saved_contexts;
        let end = self.node_pos();
        Some(self.new_node_list(pos, end, list))
    }

    /// Parses a delimited list between `opening` and `closing`, or a missing list when
    /// `opening` is absent. Returns `None` when an element fails to parse.
    pub(super) fn parse_bracketed_list(
        &mut self,
        kind: ParsingContext,
        parse_element: impl FnMut(&mut Self) -> Option<NodeId>,
        opening: SyntaxKind,
        closing: SyntaxKind,
    ) -> Option<NodeList> {
        if self.parse_expected(opening) {
            let result = self.parse_delimited_list(kind, parse_element);
            self.parse_expected(closing);
            return result;
        }
        let pos = self.node_pos();
        Some(self.builder.add_missing_list(super::to_u32(pos)))
    }

    fn abort_parsing_list_or_move_to_next_token(&mut self, kind: ParsingContext) -> bool {
        self.parsing_context_errors(kind);
        if self.is_in_some_parsing_context() {
            return true;
        }
        self.next_token();
        false
    }

    /// Returns whether the current token is an element or terminator of any enclosing list.
    fn is_in_some_parsing_context(&mut self) -> bool {
        assert!(self.parsing_contexts != 0, "missing parsing context");
        ParsingContext::ALL.into_iter().any(|kind| {
            self.parsing_contexts & kind.bit() != 0
                && (self.is_list_element(kind, true) || self.is_list_terminator(kind))
        })
    }

    fn parsing_context_errors(&mut self, context: ParsingContext) {
        use ParsingContext as Context;
        let message = match context {
            Context::SourceElements if self.token == SyntaxKind::DefaultKeyword => {
                return self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &["export"]);
            }
            Context::SourceElements | Context::BlockStatements => {
                diagnostics::DECLARATION_OR_STATEMENT_EXPECTED
            }
            Context::SwitchClauses => diagnostics::X_CASE_OR_DEFAULT_EXPECTED,
            Context::SwitchClauseStatements => diagnostics::STATEMENT_EXPECTED,
            Context::RestProperties | Context::TypeMembers => {
                diagnostics::PROPERTY_OR_SIGNATURE_EXPECTED
            }
            Context::ClassMembers => {
                diagnostics::UNEXPECTED_TOKEN_A_CONSTRUCTOR_METHOD_ACCESSOR_OR_PROPERTY_WAS_EXPECTED
            }
            Context::EnumMembers => diagnostics::ENUM_MEMBER_EXPECTED,
            Context::HeritageClauseElement => diagnostics::EXPRESSION_EXPECTED,
            Context::VariableDeclarations if self.token.is_keyword_kind() => {
                let token = token_to_string(self.token);
                return self.parse_error_at_current_token(
                    diagnostics::X_0_IS_NOT_ALLOWED_AS_A_VARIABLE_DECLARATION_NAME,
                    &[token],
                );
            }
            Context::VariableDeclarations => diagnostics::VARIABLE_DECLARATION_EXPECTED,
            Context::ObjectBindingElements => diagnostics::PROPERTY_DESTRUCTURING_PATTERN_EXPECTED,
            Context::ArrayBindingElements => {
                diagnostics::ARRAY_ELEMENT_DESTRUCTURING_PATTERN_EXPECTED
            }
            Context::ArgumentExpressions => diagnostics::ARGUMENT_EXPRESSION_EXPECTED,
            Context::ObjectLiteralMembers => diagnostics::PROPERTY_ASSIGNMENT_EXPECTED,
            Context::ArrayLiteralMembers => diagnostics::EXPRESSION_OR_COMMA_EXPECTED,
            Context::Parameters if self.token.is_keyword_kind() => {
                let token = token_to_string(self.token);
                return self.parse_error_at_current_token(
                    diagnostics::X_0_IS_NOT_ALLOWED_AS_A_PARAMETER_NAME,
                    &[token],
                );
            }
            Context::Parameters | Context::JsdocParameters => {
                diagnostics::PARAMETER_DECLARATION_EXPECTED
            }
            Context::TypeParameters => diagnostics::TYPE_PARAMETER_DECLARATION_EXPECTED,
            Context::TypeArguments => diagnostics::TYPE_ARGUMENT_EXPECTED,
            Context::TupleElementTypes => diagnostics::TYPE_EXPECTED,
            Context::HeritageClauses => diagnostics::UNEXPECTED_TOKEN_EXPECTED,
            Context::ImportOrExportSpecifiers if self.token == SyntaxKind::FromKeyword => {
                return self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &["}"]);
            }
            Context::ImportOrExportSpecifiers
            | Context::JsxAttributes
            | Context::JsxChildren
            | Context::JsdocComment => diagnostics::IDENTIFIER_EXPECTED,
            Context::ImportAttributes => diagnostics::IDENTIFIER_OR_STRING_LITERAL_EXPECTED,
        };
        self.parse_error_at_current_token(message, &[]);
    }

    fn is_list_element(&mut self, context: ParsingContext, in_error_recovery: bool) -> bool {
        use ParsingContext as Context;
        match context {
            Context::SourceElements
            | Context::BlockStatements
            | Context::SwitchClauseStatements => {
                !(self.token == SyntaxKind::SemicolonToken && in_error_recovery)
                    && self.is_start_of_statement()
            }
            Context::SwitchClauses => {
                matches!(
                    self.token,
                    SyntaxKind::CaseKeyword | SyntaxKind::DefaultKeyword
                )
            }
            Context::TypeMembers => self.look_ahead(Self::scan_type_member_start),
            Context::ClassMembers => {
                self.look_ahead(Self::scan_class_member_start)
                    || (self.token == SyntaxKind::SemicolonToken && !in_error_recovery)
            }
            Context::EnumMembers => {
                self.token == SyntaxKind::OpenBracketToken || self.is_literal_property_name()
            }
            Context::ObjectLiteralMembers => {
                matches!(
                    self.token,
                    SyntaxKind::OpenBracketToken
                        | SyntaxKind::AsteriskToken
                        | SyntaxKind::DotDotDotToken
                        | SyntaxKind::DotToken
                ) || self.is_literal_property_name()
            }
            Context::RestProperties => self.is_literal_property_name(),
            Context::ObjectBindingElements => {
                matches!(
                    self.token,
                    SyntaxKind::OpenBracketToken | SyntaxKind::DotDotDotToken
                ) || self.is_literal_property_name()
            }
            Context::ImportAttributes => self.is_import_attribute_name(),
            Context::HeritageClauseElement => {
                if self.token == SyntaxKind::OpenBraceToken {
                    return self.is_valid_heritage_clause_object_literal();
                }
                if in_error_recovery {
                    self.is_identifier() && !self.is_heritage_clause_extends_or_implements_keyword()
                } else {
                    self.is_start_of_left_hand_side_expression()
                        && !self.is_heritage_clause_extends_or_implements_keyword()
                }
            }
            Context::VariableDeclarations => {
                self.is_binding_identifier_or_private_identifier_or_pattern()
            }
            Context::ArrayBindingElements => {
                matches!(
                    self.token,
                    SyntaxKind::CommaToken | SyntaxKind::DotDotDotToken
                ) || self.is_binding_identifier_or_private_identifier_or_pattern()
            }
            Context::TypeParameters => {
                matches!(self.token, SyntaxKind::InKeyword | SyntaxKind::ConstKeyword)
                    || self.is_identifier()
            }
            Context::ArrayLiteralMembers
                if matches!(self.token, SyntaxKind::CommaToken | SyntaxKind::DotToken) =>
            {
                true
            }
            Context::ArrayLiteralMembers | Context::ArgumentExpressions => {
                self.token == SyntaxKind::DotDotDotToken || self.is_start_of_expression()
            }
            Context::Parameters => self.is_start_of_parameter(false),
            Context::JsdocParameters => self.is_start_of_parameter(true),
            Context::TypeArguments | Context::TupleElementTypes => {
                self.token == SyntaxKind::CommaToken || self.is_start_of_type(false)
            }
            Context::HeritageClauses => self.is_heritage_clause(),
            Context::ImportOrExportSpecifiers => {
                if self.token == SyntaxKind::FromKeyword
                    && self.look_ahead(Self::next_token_is_token_string_literal)
                {
                    return false;
                }
                self.token == SyntaxKind::StringLiteral || self.token.is_identifier_or_keyword()
            }
            Context::JsxAttributes => {
                self.token.is_identifier_or_keyword() || self.token == SyntaxKind::OpenBraceToken
            }
            Context::JsxChildren | Context::JsdocComment => true,
        }
    }

    pub(super) fn is_list_terminator(&mut self, kind: ParsingContext) -> bool {
        use ParsingContext as Context;
        if self.token == SyntaxKind::EndOfFile {
            return true;
        }
        match kind {
            Context::BlockStatements
            | Context::SwitchClauses
            | Context::TypeMembers
            | Context::ClassMembers
            | Context::EnumMembers
            | Context::ObjectLiteralMembers
            | Context::ObjectBindingElements
            | Context::ImportOrExportSpecifiers
            | Context::ImportAttributes => self.token == SyntaxKind::CloseBraceToken,
            Context::SwitchClauseStatements => matches!(
                self.token,
                SyntaxKind::CloseBraceToken | SyntaxKind::CaseKeyword | SyntaxKind::DefaultKeyword
            ),
            Context::HeritageClauseElement => matches!(
                self.token,
                SyntaxKind::OpenBraceToken
                    | SyntaxKind::ExtendsKeyword
                    | SyntaxKind::ImplementsKeyword
            ),
            Context::VariableDeclarations => {
                self.can_parse_semicolon()
                    || matches!(
                        self.token,
                        SyntaxKind::InKeyword
                            | SyntaxKind::OfKeyword
                            | SyntaxKind::EqualsGreaterThanToken
                    )
            }
            Context::TypeParameters => matches!(
                self.token,
                SyntaxKind::GreaterThanToken
                    | SyntaxKind::OpenParenToken
                    | SyntaxKind::OpenBraceToken
                    | SyntaxKind::ExtendsKeyword
                    | SyntaxKind::ImplementsKeyword
            ),
            Context::ArgumentExpressions => matches!(
                self.token,
                SyntaxKind::CloseParenToken | SyntaxKind::SemicolonToken
            ),
            Context::ArrayLiteralMembers
            | Context::TupleElementTypes
            | Context::ArrayBindingElements => self.token == SyntaxKind::CloseBracketToken,
            Context::JsdocParameters | Context::Parameters | Context::RestProperties => matches!(
                self.token,
                SyntaxKind::CloseParenToken | SyntaxKind::CloseBracketToken
            ),
            Context::TypeArguments => self.token != SyntaxKind::CommaToken,
            Context::HeritageClauses => matches!(
                self.token,
                SyntaxKind::OpenBraceToken | SyntaxKind::CloseBraceToken
            ),
            Context::JsxAttributes => matches!(
                self.token,
                SyntaxKind::GreaterThanToken | SyntaxKind::SlashToken
            ),
            Context::JsxChildren => {
                self.token == SyntaxKind::LessThanToken
                    && self.look_ahead(|parser| parser.next_token() == SyntaxKind::SlashToken)
            }
            Context::SourceElements | Context::JsdocComment => false,
        }
    }
}
