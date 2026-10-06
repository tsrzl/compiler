//! Statement parsing.

use crate::ast::{
    Block, ExpressionStatement, LabeledStatement, MissingDeclaration, NodeData, NodeId, SyntaxKind,
};
use crate::diagnostics::{self, Message};

use crate::scanner::all_keywords;
use crate::spelling::spelling_suggestion;

use super::{Parser, ParsingContext, to_u32, token_to_string};

/// Keywords longer than two characters, which TypeScript-Go offers as spelling suggestions.
fn viable_keyword_suggestions() -> Vec<&'static str> {
    all_keywords().filter(|keyword| keyword.len() > 2).collect()
}

/// Suggests a missing space after a keyword prefix, such as `declare module` for
/// `declaremodule`. TypeScript-Go checks keywords in map order; sorted order keeps it stable.
fn space_suggestion(expression_text: &str, keywords: &[&str]) -> Option<String> {
    keywords.iter().find_map(|keyword| {
        (expression_text.len() > keyword.len() + 2 && expression_text.starts_with(keyword))
            .then(|| format!("{keyword} {}", &expression_text[keyword.len()..]))
    })
}

impl Parser<'_> {
    pub(super) fn parse_statement(&mut self) -> NodeId {
        let token = self.token;
        match token {
            SyntaxKind::SemicolonToken => return self.parse_empty_statement(),
            SyntaxKind::OpenBraceToken => return self.parse_block(false, None),
            SyntaxKind::VarKeyword => {
                let (pos, jsdoc) = (self.node_pos(), self.jsdoc_scanner_info());
                return self.parse_variable_statement(pos, jsdoc, None);
            }
            SyntaxKind::LetKeyword if self.is_let_declaration() => {
                let (pos, jsdoc) = (self.node_pos(), self.jsdoc_scanner_info());
                return self.parse_variable_statement(pos, jsdoc, None);
            }
            SyntaxKind::AwaitKeyword if self.is_await_using_declaration() => {
                let (pos, jsdoc) = (self.node_pos(), self.jsdoc_scanner_info());
                return self.parse_variable_statement(pos, jsdoc, None);
            }
            SyntaxKind::UsingKeyword if self.is_using_declaration() => {
                let (pos, jsdoc) = (self.node_pos(), self.jsdoc_scanner_info());
                return self.parse_variable_statement(pos, jsdoc, None);
            }
            SyntaxKind::FunctionKeyword => {
                let (pos, jsdoc) = (self.node_pos(), self.jsdoc_scanner_info());
                return self.parse_function_declaration(pos, jsdoc, None);
            }
            SyntaxKind::ClassKeyword => return self.parse_unported_statement(),
            SyntaxKind::IfKeyword => return self.parse_if_statement(),
            SyntaxKind::DoKeyword => return self.parse_do_statement(),
            SyntaxKind::WhileKeyword => return self.parse_while_statement(),
            SyntaxKind::ForKeyword => return self.parse_for_or_for_in_or_for_of_statement(),
            SyntaxKind::ContinueKeyword => {
                return self.parse_break_or_continue_statement(SyntaxKind::ContinueStatement);
            }
            SyntaxKind::BreakKeyword => {
                return self.parse_break_or_continue_statement(SyntaxKind::BreakStatement);
            }
            SyntaxKind::ReturnKeyword => return self.parse_return_statement(),
            SyntaxKind::WithKeyword => return self.parse_with_statement(),
            SyntaxKind::SwitchKeyword => return self.parse_switch_statement(),
            SyntaxKind::ThrowKeyword => return self.parse_throw_statement(),
            // `catch` and `finally` without `try` are parsed as a try statement and reported.
            SyntaxKind::TryKeyword | SyntaxKind::CatchKeyword | SyntaxKind::FinallyKeyword => {
                return self.parse_try_statement();
            }
            SyntaxKind::DebuggerKeyword => return self.parse_debugger_statement(),
            SyntaxKind::AtToken => return self.parse_declaration(),
            SyntaxKind::AsyncKeyword
            | SyntaxKind::InterfaceKeyword
            | SyntaxKind::TypeKeyword
            | SyntaxKind::ModuleKeyword
            | SyntaxKind::NamespaceKeyword
            | SyntaxKind::DeclareKeyword
            | SyntaxKind::ConstKeyword
            | SyntaxKind::EnumKeyword
            | SyntaxKind::ExportKeyword
            | SyntaxKind::ImportKeyword
            | SyntaxKind::PrivateKeyword
            | SyntaxKind::ProtectedKeyword
            | SyntaxKind::PublicKeyword
            | SyntaxKind::AbstractKeyword
            | SyntaxKind::AccessorKeyword
            | SyntaxKind::StaticKeyword
            | SyntaxKind::ReadonlyKeyword
            | SyntaxKind::GlobalKeyword
                if self.is_start_of_declaration() =>
            {
                return self.parse_declaration();
            }
            _ => {}
        }
        self.parse_expression_or_labeled_statement()
    }

    /// identifier followed by `:`.
    /// Parses an expression statement, or a labeled statement when the expression is an
    /// identifier followed by `:`.
    fn parse_expression_or_labeled_statement(&mut self) -> NodeId {
        let pos = self.node_pos();
        let mut jsdoc = self.jsdoc_scanner_info();
        let has_paren = self.token == SyntaxKind::OpenParenToken;
        let expression = self.parse_expression();
        if self.builder.node(expression).kind() == SyntaxKind::Identifier
            && self.parse_optional(SyntaxKind::ColonToken)
        {
            let statement = self.parse_statement();
            let result = self.finish_node(
                SyntaxKind::LabeledStatement,
                pos,
                NodeData::LabeledStatement(LabeledStatement {
                    label: expression,
                    statement,
                }),
            );
            self.with_jsdoc(result, jsdoc);
            return result;
        }
        if !self.try_parse_semicolon() {
            self.parse_error_for_missing_semicolon_after(expression);
        }
        let result = self.finish_node(
            SyntaxKind::ExpressionStatement,
            pos,
            NodeData::ExpressionStatement(ExpressionStatement { expression }),
        );
        if has_paren {
            jsdoc.has_jsdoc = false;
        }
        self.with_jsdoc(result, jsdoc);
        result
    }

    pub(super) fn parse_error_for_missing_semicolon_after(&mut self, node: NodeId) {
        let node_ref = self.builder.node(node);
        if let Some(tagged) = node_ref.data().as_tagged_template_expression() {
            // `module `M1` {` parses as a tagged template.
            let template = self.builder.node(tagged.template);
            let (start, end) = (
                self.skip_trivia(template.pos() as usize),
                template.end() as usize,
            );
            self.parse_error_at(
                start,
                end,
                diagnostics::MODULE_DECLARATION_NAMES_MAY_ONLY_USE_OR_QUOTED_STRINGS,
                &[],
            );
            return;
        }
        let expression_text = node_ref
            .data()
            .as_identifier()
            .map_or_else(String::new, |identifier| identifier.text.to_string());
        if expression_text.is_empty() {
            self.parse_error_at_current_token(
                diagnostics::X_0_EXPECTED,
                &[token_to_string(SyntaxKind::SemicolonToken)],
            );
            return;
        }
        let pos = self.skip_trivia(node_ref.pos() as usize);
        let end = node_ref.end() as usize;
        match expression_text.as_str() {
            "const" | "let" | "var" => {
                self.parse_error_at(
                    pos,
                    end,
                    diagnostics::VARIABLE_DECLARATION_NOT_ALLOWED_AT_THIS_LOCATION,
                    &[],
                );
                return;
            }
            "declare" => return,
            "interface" => {
                self.parse_error_for_invalid_name(
                    diagnostics::INTERFACE_NAME_CANNOT_BE_0,
                    diagnostics::INTERFACE_MUST_BE_GIVEN_A_NAME,
                    SyntaxKind::OpenBraceToken,
                );
                return;
            }
            "is" => {
                let token_start = self.scanner.token_start();
                self.parse_error_at(
                    pos,
                    token_start,
                    diagnostics::A_TYPE_PREDICATE_IS_ONLY_ALLOWED_IN_RETURN_TYPE_POSITION_FOR_FUNCTIONS_AND_METHODS,
                    &[],
                );
                return;
            }
            "module" | "namespace" => {
                self.parse_error_for_invalid_name(
                    diagnostics::NAMESPACE_NAME_CANNOT_BE_0,
                    diagnostics::NAMESPACE_MUST_BE_GIVEN_A_NAME,
                    SyntaxKind::OpenBraceToken,
                );
                return;
            }
            "type" => {
                self.parse_error_for_invalid_name(
                    diagnostics::TYPE_ALIAS_NAME_CANNOT_BE_0,
                    diagnostics::TYPE_ALIAS_MUST_BE_GIVEN_A_NAME,
                    SyntaxKind::EqualsToken,
                );
                return;
            }
            _ => {}
        }
        // A misspelled keyword, or a keyword missing its following space.
        let keywords = viable_keyword_suggestions();
        let suggestion = spelling_suggestion(&expression_text, keywords.iter().copied())
            .map(str::to_owned)
            .or_else(|| space_suggestion(&expression_text, &keywords));
        if let Some(suggestion) = suggestion {
            self.parse_error_at(
                pos,
                end,
                diagnostics::UNKNOWN_KEYWORD_OR_IDENTIFIER_DID_YOU_MEAN_0,
                &[&suggestion],
            );
            return;
        }
        // Unknown tokens were already reported by the scanner.
        if self.token == SyntaxKind::Unknown {
            return;
        }
        self.parse_error_at(pos, end, diagnostics::UNEXPECTED_KEYWORD_OR_IDENTIFIER, &[]);
    }

    fn parse_error_for_invalid_name(
        &mut self,
        name_message: Message,
        blank_message: Message,
        token_if_blank_name: SyntaxKind,
    ) {
        if self.token == token_if_blank_name {
            self.parse_error_at_current_token(blank_message, &[]);
        } else {
            let value = self.scanner.token_value().to_owned();
            self.parse_error_at_current_token(name_message, &[&value]);
        }
    }

    /// Reports and skips a statement whose syntax has not been ported from TypeScript-Go yet.
    ///
    /// This keeps unported syntax visible as TS1128 instead of producing a plausible tree; it is
    /// removed when statement parsing is complete.
    pub(super) fn parse_unported_statement(&mut self) -> NodeId {
        let pos = self.node_pos();
        self.parse_error_at_current_token(diagnostics::DECLARATION_OR_STATEMENT_EXPECTED, &[]);
        self.next_token();
        self.finish_node(
            SyntaxKind::MissingDeclaration,
            pos,
            NodeData::MissingDeclaration(MissingDeclaration { modifiers: None }),
        )
    }

    fn parse_empty_statement(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        self.parse_expected(SyntaxKind::SemicolonToken);
        let result = self.finish_node(SyntaxKind::EmptyStatement, pos, NodeData::EmptyStatement);
        self.with_jsdoc(result, jsdoc);
        result
    }

    pub(super) fn parse_block(
        &mut self,
        ignore_missing_open_brace: bool,
        message: Option<Message>,
    ) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        let open_brace_position = self.scanner.token_start();
        let open_brace_parsed =
            self.parse_expected_with_diagnostic(SyntaxKind::OpenBraceToken, message, true);
        let (statements, multi_line) = if open_brace_parsed || ignore_missing_open_brace {
            let multi_line = self.has_preceding_line_break();
            let statements =
                self.parse_list(ParsingContext::BlockStatements, Self::parse_statement);
            self.parse_expected_matching_brackets(
                SyntaxKind::OpenBraceToken,
                SyntaxKind::CloseBraceToken,
                open_brace_parsed,
                open_brace_position,
            );
            (statements, multi_line)
        } else {
            (
                self.builder.add_missing_list(to_u32(self.node_pos())),
                false,
            )
        };
        let result = self.finish_node(
            SyntaxKind::Block,
            pos,
            NodeData::Block(Block {
                statements,
                multi_line,
            }),
        );
        self.with_jsdoc(result, jsdoc);
        if (open_brace_parsed || ignore_missing_open_brace) && self.token == SyntaxKind::EqualsToken
        {
            self.parse_error_at_current_token(
                diagnostics::DECLARATION_OR_STATEMENT_EXPECTED_THIS_FOLLOWS_A_BLOCK_OF_STATEMENTS_SO_IF_YOU_INTENDED_TO_WRITE_A_DESTRUCTURING_ASSIGNMENT_YOU_MIGHT_NEED_TO_WRAP_THE_WHOLE_ASSIGNMENT_IN_PARENTHESES,
                &[],
            );
            self.next_token();
        }
        result
    }

    pub(super) fn parse_expected_matching_brackets(
        &mut self,
        open_kind: SyntaxKind,
        close_kind: SyntaxKind,
        open_parsed: bool,
        open_position: usize,
    ) {
        if self.token == close_kind {
            self.next_token();
            return;
        }
        let reported_before = self.diagnostics.len();
        self.parse_error_at_current_token(
            diagnostics::X_0_EXPECTED,
            &[token_to_string(close_kind)],
        );
        if !open_parsed || self.diagnostics.len() == reported_before {
            return;
        }
        let related = super::ParseDiagnostic::new(
            diagnostics::THE_PARSER_EXPECTED_TO_FIND_A_1_TO_MATCH_THE_0_TOKEN_HERE,
            open_position,
            open_position,
            &[token_to_string(open_kind), token_to_string(close_kind)],
        );
        if let Some(last) = self.diagnostics.last_mut() {
            last.related.push(related);
        }
    }
}
