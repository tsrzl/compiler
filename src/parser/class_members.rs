//! Class and object-literal members.

use crate::ast::{MethodDeclaration, ModifierFlags, ModifierList, NodeData, NodeId, SyntaxKind};
use crate::diagnostics::Message;

use super::signatures::SignatureFlags;
use super::{JsdocScannerInfo, Parser};

impl Parser<'_> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn parse_method_declaration(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
        asterisk_token: Option<NodeId>,
        name: NodeId,
        postfix_token: Option<NodeId>,
        message: Option<Message>,
    ) -> NodeId {
        let is_async =
            modifiers.is_some_and(|modifiers| modifiers.flags().intersects(ModifierFlags::ASYNC));
        let flags = SignatureFlags::function(asterisk_token.is_some(), is_async);
        let type_parameters = self.parse_type_parameters();
        let parameters = self.parse_parameters(flags);
        let type_node = self.parse_return_type(SyntaxKind::ColonToken, false);
        let body = self.parse_function_block_or_semicolon(flags, message);
        let result = self.finish_node(
            SyntaxKind::MethodDeclaration,
            pos,
            NodeData::MethodDeclaration(MethodDeclaration {
                modifiers,
                asterisk_token,
                name,
                postfix_token,
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
}
