//! Interfaces, type aliases, and enums.

use crate::ast::{
    EnumDeclaration, EnumMember, InterfaceDeclaration, ModifierList, NodeData, NodeFlags, NodeId,
    SyntaxKind, TypeAliasDeclaration,
};
use crate::diagnostics;

use super::{JsdocScannerInfo, Parser, ParsingContext};

impl Parser<'_> {
    pub(super) fn parse_interface_declaration(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
    ) -> NodeId {
        self.parse_expected(SyntaxKind::InterfaceKeyword);
        let name = self.parse_identifier();
        let type_parameters = self.parse_type_parameters();
        let heritage_clauses = self.parse_heritage_clauses();
        let members = self.parse_object_type_members();
        let result = self.finish_node(
            SyntaxKind::InterfaceDeclaration,
            pos,
            NodeData::InterfaceDeclaration(InterfaceDeclaration {
                modifiers,
                name,
                type_parameters,
                heritage_clauses,
                members,
            }),
        );
        self.with_jsdoc(result, jsdoc);
        result
    }

    pub(super) fn parse_type_alias_declaration(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
    ) -> NodeId {
        self.parse_expected(SyntaxKind::TypeKeyword);
        if self.has_preceding_line_break() {
            self.parse_error_at_current_token(diagnostics::LINE_BREAK_NOT_PERMITTED_HERE, &[]);
        }
        let name = self.parse_identifier();
        let type_parameters = self.parse_type_parameters();
        self.parse_expected(SyntaxKind::EqualsToken);
        // `type T = intrinsic` declares a compiler-provided type unless `intrinsic` is qualified.
        let type_node = if self.token == SyntaxKind::IntrinsicKeyword
            && self.look_ahead(|parser| parser.next_token() != SyntaxKind::DotToken)
        {
            let pos = self.node_pos();
            self.next_token();
            self.finish_node(SyntaxKind::IntrinsicKeyword, pos, NodeData::KeywordTypeNode)
        } else {
            self.parse_type()
        };
        self.parse_semicolon();
        let result = self.finish_node(
            SyntaxKind::TypeAliasDeclaration,
            pos,
            NodeData::TypeAliasDeclaration(TypeAliasDeclaration {
                modifiers,
                name,
                type_parameters,
                type_node,
            }),
        );
        self.with_jsdoc(result, jsdoc);
        result
    }

    fn parse_enum_member(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        let name = self.parse_property_name();
        let initializer = self.do_in_context(
            NodeFlags::DISALLOW_IN_CONTEXT,
            false,
            Self::parse_initializer,
        );
        let result = self.finish_node(
            SyntaxKind::EnumMember,
            pos,
            NodeData::EnumMember(EnumMember { name, initializer }),
        );
        self.with_jsdoc(result, jsdoc);
        result
    }

    pub(super) fn parse_enum_declaration(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
    ) -> NodeId {
        let saved_has_await_identifier = self.statement_has_await_identifier;
        self.parse_expected(SyntaxKind::EnumKeyword);
        let name = self.parse_identifier();
        let members = if self.parse_expected(SyntaxKind::OpenBraceToken) {
            let members = self.do_in_context(
                NodeFlags::YIELD_CONTEXT | NodeFlags::AWAIT_CONTEXT,
                false,
                |parser| {
                    parser.parse_delimited_list(ParsingContext::EnumMembers, |parser| {
                        Some(parser.parse_enum_member())
                    })
                },
            );
            self.parse_expected(SyntaxKind::CloseBraceToken);
            members.expect("enum members always parse")
        } else {
            let pos = self.node_pos();
            self.builder.add_missing_list(super::to_u32(pos))
        };
        let result = self.finish_node(
            SyntaxKind::EnumDeclaration,
            pos,
            NodeData::EnumDeclaration(EnumDeclaration {
                modifiers,
                name,
                members,
            }),
        );
        self.with_jsdoc(result, jsdoc);
        self.statement_has_await_identifier = saved_has_await_identifier;
        result
    }
}
