//! Statement parsing.

use crate::ast::{Block, MissingDeclaration, NodeData, NodeId, SyntaxKind};
use crate::diagnostics::{self, Message};

use super::{Parser, ParsingContext, to_u32, token_to_string};

impl Parser<'_> {
    pub(super) fn parse_statement(&mut self) -> NodeId {
        match self.token {
            SyntaxKind::SemicolonToken => self.parse_empty_statement(),
            SyntaxKind::OpenBraceToken => self.parse_block(false, None),
            _ => self.parse_unported_statement(),
        }
    }

    /// Reports and skips a statement whose syntax has not been ported from TypeScript-Go yet.
    ///
    /// This keeps unported syntax visible as TS1128 instead of producing a plausible tree; it is
    /// removed when statement parsing is complete.
    fn parse_unported_statement(&mut self) -> NodeId {
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
