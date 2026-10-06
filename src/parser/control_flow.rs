//! Control-flow statements.

use crate::ast::{
    BreakStatement, CaseBlock, CaseOrDefaultClause, CatchClause, ContinueStatement, DoStatement,
    ForInOrOfStatement, ForStatement, IfStatement, NodeData, NodeFlags, NodeId, ReturnStatement,
    SwitchStatement, SyntaxKind, ThrowStatement, TryStatement, WhileStatement, WithStatement,
};
use crate::diagnostics;

use super::{Parser, ParsingContext};

impl Parser<'_> {
    /// Parses `( expression )` after a statement keyword, relating a missing `)` to its `(`.
    fn parse_parenthesized_condition(&mut self) -> NodeId {
        let open_paren_position = self.scanner.token_start();
        let open_paren_parsed = self.parse_expected(SyntaxKind::OpenParenToken);
        let expression = self.parse_expression_allow_in();
        self.parse_expected_matching_brackets(
            SyntaxKind::OpenParenToken,
            SyntaxKind::CloseParenToken,
            open_paren_parsed,
            open_paren_position,
        );
        expression
    }

    /// Finishes a statement node and attaches the `JSDoc` that preceded it.
    fn finish_statement(
        &mut self,
        kind: SyntaxKind,
        pos: usize,
        jsdoc: super::JsdocScannerInfo,
        data: NodeData,
    ) -> NodeId {
        let result = self.finish_node(kind, pos, data);
        self.with_jsdoc(result, jsdoc);
        result
    }

    pub(super) fn parse_if_statement(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        self.parse_expected(SyntaxKind::IfKeyword);
        let expression = self.parse_parenthesized_condition();
        let then_statement = self.parse_statement();
        let else_statement = self
            .parse_optional(SyntaxKind::ElseKeyword)
            .then(|| self.parse_statement());
        self.finish_statement(
            SyntaxKind::IfStatement,
            pos,
            jsdoc,
            NodeData::IfStatement(IfStatement {
                expression,
                then_statement,
                else_statement,
            }),
        )
    }

    pub(super) fn parse_do_statement(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        self.parse_expected(SyntaxKind::DoKeyword);
        let statement = self.parse_statement();
        self.parse_expected(SyntaxKind::WhileKeyword);
        let expression = self.parse_parenthesized_condition();
        // `do;while(0)x` inserts a semicolon before `x`, per de facto engine behavior.
        self.parse_optional(SyntaxKind::SemicolonToken);
        self.finish_statement(
            SyntaxKind::DoStatement,
            pos,
            jsdoc,
            NodeData::DoStatement(DoStatement {
                statement,
                expression,
            }),
        )
    }

    pub(super) fn parse_while_statement(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        self.parse_expected(SyntaxKind::WhileKeyword);
        let expression = self.parse_parenthesized_condition();
        let statement = self.parse_statement();
        self.finish_statement(
            SyntaxKind::WhileStatement,
            pos,
            jsdoc,
            NodeData::WhileStatement(WhileStatement {
                expression,
                statement,
            }),
        )
    }

    pub(super) fn parse_for_or_for_in_or_for_of_statement(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        self.parse_expected(SyntaxKind::ForKeyword);
        let await_token = self.parse_optional_token(SyntaxKind::AwaitKeyword);
        self.parse_expected(SyntaxKind::OpenParenToken);
        let initializer =
            (self.token != SyntaxKind::SemicolonToken).then(|| self.parse_for_initializer());
        let is_for_of = if await_token.is_some() {
            self.parse_expected(SyntaxKind::OfKeyword)
        } else {
            self.parse_optional(SyntaxKind::OfKeyword)
        };
        let (kind, data) = if is_for_of {
            let expression = self.do_in_context(
                NodeFlags::DISALLOW_IN_CONTEXT,
                false,
                Self::parse_assignment_expression_or_higher,
            );
            self.parse_expected(SyntaxKind::CloseParenToken);
            let statement = self.parse_statement();
            (
                SyntaxKind::ForOfStatement,
                Self::for_in_or_of(await_token, initializer, expression, statement),
            )
        } else if self.parse_optional(SyntaxKind::InKeyword) {
            let expression = self.parse_expression_allow_in();
            self.parse_expected(SyntaxKind::CloseParenToken);
            let statement = self.parse_statement();
            (
                SyntaxKind::ForInStatement,
                Self::for_in_or_of(None, initializer, expression, statement),
            )
        } else {
            self.parse_expected(SyntaxKind::SemicolonToken);
            let condition = (!matches!(
                self.token,
                SyntaxKind::SemicolonToken | SyntaxKind::CloseParenToken
            ))
            .then(|| self.parse_expression_allow_in());
            self.parse_expected(SyntaxKind::SemicolonToken);
            let incrementor = (self.token != SyntaxKind::CloseParenToken)
                .then(|| self.parse_expression_allow_in());
            self.parse_expected(SyntaxKind::CloseParenToken);
            let statement = self.parse_statement();
            (
                SyntaxKind::ForStatement,
                NodeData::ForStatement(ForStatement {
                    initializer,
                    condition,
                    incrementor,
                    statement,
                }),
            )
        };
        self.finish_statement(kind, pos, jsdoc, data)
    }

    fn for_in_or_of(
        await_modifier: Option<NodeId>,
        initializer: Option<NodeId>,
        expression: NodeId,
        statement: NodeId,
    ) -> NodeData {
        let initializer = initializer.expect("for-in and for-of statements have an initializer");
        NodeData::ForInOrOfStatement(ForInOrOfStatement {
            await_modifier,
            initializer,
            expression,
            statement,
        })
    }

    fn parse_for_initializer(&mut self) -> NodeId {
        let starts_declaration = match self.token {
            SyntaxKind::VarKeyword | SyntaxKind::LetKeyword | SyntaxKind::ConstKeyword => true,
            SyntaxKind::UsingKeyword => self.look_ahead(|parser| {
                parser.next_token_is_binding_identifier_or_start_of_destructuring_on_same_line(true)
            }),
            SyntaxKind::AwaitKeyword => self.look_ahead(|parser| {
                parser.next_token() == SyntaxKind::UsingKeyword
                    && parser
                        .next_token_is_binding_identifier_or_start_of_destructuring_on_same_line(
                            false,
                        )
            }),
            _ => false,
        };
        if starts_declaration {
            self.parse_variable_declaration_list(true)
        } else {
            self.do_in_context(NodeFlags::DISALLOW_IN_CONTEXT, true, Self::parse_expression)
        }
    }

    pub(super) fn parse_break_or_continue_statement(&mut self, kind: SyntaxKind) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        let keyword = if kind == SyntaxKind::BreakStatement {
            SyntaxKind::BreakKeyword
        } else {
            SyntaxKind::ContinueKeyword
        };
        self.parse_expected(keyword);
        let label = (!self.can_parse_semicolon()).then(|| self.parse_identifier());
        self.parse_semicolon();
        let data = if kind == SyntaxKind::BreakStatement {
            NodeData::BreakStatement(BreakStatement { label })
        } else {
            NodeData::ContinueStatement(ContinueStatement { label })
        };
        self.finish_statement(kind, pos, jsdoc, data)
    }

    pub(super) fn parse_return_statement(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        self.parse_expected(SyntaxKind::ReturnKeyword);
        let expression = (!self.can_parse_semicolon()).then(|| self.parse_expression_allow_in());
        self.parse_semicolon();
        self.finish_statement(
            SyntaxKind::ReturnStatement,
            pos,
            jsdoc,
            NodeData::ReturnStatement(ReturnStatement { expression }),
        )
    }

    pub(super) fn parse_with_statement(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        self.parse_expected(SyntaxKind::WithKeyword);
        let expression = self.parse_parenthesized_condition();
        let statement =
            self.do_in_context(NodeFlags::IN_WITH_STATEMENT, true, Self::parse_statement);
        self.finish_statement(
            SyntaxKind::WithStatement,
            pos,
            jsdoc,
            NodeData::WithStatement(WithStatement {
                expression,
                statement,
            }),
        )
    }

    fn parse_case_or_default_clause(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        let (kind, expression) = if self.token == SyntaxKind::CaseKeyword {
            self.parse_expected(SyntaxKind::CaseKeyword);
            (
                SyntaxKind::CaseClause,
                Some(self.parse_expression_allow_in()),
            )
        } else {
            self.parse_expected(SyntaxKind::DefaultKeyword);
            (SyntaxKind::DefaultClause, None)
        };
        self.parse_expected(SyntaxKind::ColonToken);
        let statements = self.parse_list(
            ParsingContext::SwitchClauseStatements,
            Self::parse_statement,
        );
        self.finish_statement(
            kind,
            pos,
            jsdoc,
            NodeData::CaseOrDefaultClause(CaseOrDefaultClause {
                expression,
                statements,
            }),
        )
    }

    pub(super) fn parse_switch_statement(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        self.parse_expected(SyntaxKind::SwitchKeyword);
        self.parse_expected(SyntaxKind::OpenParenToken);
        let expression = self.parse_expression_allow_in();
        self.parse_expected(SyntaxKind::CloseParenToken);
        let block_pos = self.node_pos();
        let block_jsdoc = self.jsdoc_scanner_info();
        self.parse_expected(SyntaxKind::OpenBraceToken);
        let clauses = self.parse_list(
            ParsingContext::SwitchClauses,
            Self::parse_case_or_default_clause,
        );
        self.parse_expected(SyntaxKind::CloseBraceToken);
        let case_block = self.finish_statement(
            SyntaxKind::CaseBlock,
            block_pos,
            block_jsdoc,
            NodeData::CaseBlock(CaseBlock { clauses }),
        );
        self.finish_statement(
            SyntaxKind::SwitchStatement,
            pos,
            jsdoc,
            NodeData::SwitchStatement(SwitchStatement {
                expression,
                case_block,
            }),
        )
    }

    pub(super) fn parse_throw_statement(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        self.parse_expected(SyntaxKind::ThrowKeyword);
        // A line break after `throw` is a grammar error reported later, so the expression is
        // missing rather than taken from the next line.
        let expression = if self.has_preceding_line_break() {
            self.create_missing_identifier()
        } else {
            self.parse_expression_allow_in()
        };
        if !self.try_parse_semicolon() {
            self.parse_error_for_missing_semicolon_after(expression);
        }
        self.finish_statement(
            SyntaxKind::ThrowStatement,
            pos,
            jsdoc,
            NodeData::ThrowStatement(ThrowStatement { expression }),
        )
    }

    pub(super) fn parse_try_statement(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        self.parse_expected(SyntaxKind::TryKeyword);
        let try_block = self.parse_block(false, None);
        let catch_clause =
            (self.token == SyntaxKind::CatchKeyword).then(|| self.parse_catch_clause());
        // Without a catch clause, a finally block is required, so parse one regardless.
        let finally_block = if catch_clause.is_none() || self.token == SyntaxKind::FinallyKeyword {
            self.parse_expected_with_diagnostic(
                SyntaxKind::FinallyKeyword,
                Some(diagnostics::X_CATCH_OR_FINALLY_EXPECTED),
                true,
            );
            Some(self.parse_block(false, None))
        } else {
            None
        };
        self.finish_statement(
            SyntaxKind::TryStatement,
            pos,
            jsdoc,
            NodeData::TryStatement(TryStatement {
                try_block,
                catch_clause,
                finally_block,
            }),
        )
    }

    fn parse_catch_clause(&mut self) -> NodeId {
        let pos = self.node_pos();
        self.parse_expected(SyntaxKind::CatchKeyword);
        let variable_declaration = if self.parse_optional(SyntaxKind::OpenParenToken) {
            let declaration = self.parse_variable_declaration(false);
            self.parse_expected(SyntaxKind::CloseParenToken);
            Some(declaration)
        } else {
            None
        };
        let block = self.parse_block(false, None);
        self.finish_node(
            SyntaxKind::CatchClause,
            pos,
            NodeData::CatchClause(CatchClause {
                variable_declaration,
                block,
            }),
        )
    }

    pub(super) fn parse_debugger_statement(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        self.parse_expected(SyntaxKind::DebuggerKeyword);
        self.parse_semicolon();
        self.finish_statement(
            SyntaxKind::DebuggerStatement,
            pos,
            jsdoc,
            NodeData::DebuggerStatement,
        )
    }
}
