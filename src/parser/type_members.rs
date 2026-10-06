//! Members of interfaces and type literals, and accessor declarations.

use crate::ast::{
    CallSignatureDeclaration, ConstructSignatureDeclaration, GetAccessorDeclaration,
    IndexSignatureDeclaration, MethodSignatureDeclaration, ModifierList, NodeData, NodeId,
    PropertySignatureDeclaration, SetAccessorDeclaration, SyntaxKind,
};

use super::signatures::SignatureFlags;
use super::{JsdocScannerInfo, Parser, ParsingContext};

impl Parser<'_> {
    pub(super) fn parse_type_member(&mut self) -> NodeId {
        if matches!(
            self.token,
            SyntaxKind::OpenParenToken | SyntaxKind::LessThanToken
        ) {
            return self.parse_signature_member(SyntaxKind::CallSignature);
        }
        if self.token == SyntaxKind::NewKeyword
            && self.look_ahead(|parser| {
                matches!(
                    parser.next_token(),
                    SyntaxKind::OpenParenToken | SyntaxKind::LessThanToken
                )
            })
        {
            return self.parse_signature_member(SyntaxKind::ConstructSignature);
        }
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        let modifiers = self.parse_modifiers();
        if self.parse_contextual_modifier(SyntaxKind::GetKeyword) {
            return self.parse_accessor_declaration(
                pos,
                jsdoc,
                modifiers,
                SyntaxKind::GetAccessor,
                SignatureFlags::TYPE,
            );
        }
        if self.parse_contextual_modifier(SyntaxKind::SetKeyword) {
            return self.parse_accessor_declaration(
                pos,
                jsdoc,
                modifiers,
                SyntaxKind::SetAccessor,
                SignatureFlags::TYPE,
            );
        }
        if self.is_index_signature() {
            return self.parse_index_signature_declaration(pos, jsdoc, modifiers);
        }
        self.parse_property_or_method_signature(pos, jsdoc, modifiers)
    }

    fn parse_signature_member(&mut self, kind: SyntaxKind) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        if kind == SyntaxKind::ConstructSignature {
            self.parse_expected(SyntaxKind::NewKeyword);
        }
        let type_parameters = self.parse_type_parameters();
        let parameters = self.parse_parameters(SignatureFlags::TYPE);
        let type_node = self.parse_return_type(SyntaxKind::ColonToken, true);
        self.parse_type_member_semicolon();
        let data = if kind == SyntaxKind::CallSignature {
            NodeData::CallSignatureDeclaration(CallSignatureDeclaration {
                type_parameters,
                parameters,
                type_node,
            })
        } else {
            NodeData::ConstructSignatureDeclaration(ConstructSignatureDeclaration {
                type_parameters,
                parameters,
                type_node,
            })
        };
        let result = self.finish_node(kind, pos, data);
        self.with_jsdoc(result, jsdoc);
        result
    }

    pub(super) fn parse_accessor_declaration(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
        kind: SyntaxKind,
        flags: SignatureFlags,
    ) -> NodeId {
        let name = self.parse_property_name();
        let type_parameters = self.parse_type_parameters();
        let parameters = self.parse_parameters(SignatureFlags::default());
        let type_node = self.parse_return_type(SyntaxKind::ColonToken, false);
        let body = self.parse_function_block_or_semicolon(flags, None);
        // Type parameters and setter return types are kept so the checker can report them.
        let data = if kind == SyntaxKind::GetAccessor {
            NodeData::GetAccessorDeclaration(GetAccessorDeclaration {
                modifiers,
                name,
                type_parameters,
                parameters,
                type_node,
                full_signature: None,
                body,
            })
        } else {
            NodeData::SetAccessorDeclaration(SetAccessorDeclaration {
                modifiers,
                name,
                type_parameters,
                parameters,
                type_node,
                full_signature: None,
                body,
            })
        };
        let result = self.finish_node(kind, pos, data);
        self.with_jsdoc(result, jsdoc);
        result
    }

    pub(super) fn is_index_signature(&mut self) -> bool {
        self.token == SyntaxKind::OpenBracketToken
            && self.look_ahead(Self::next_is_unambiguously_index_signature)
    }

    /// Accepts `[id:` and, for error recovery, `[...`, `[id,`, `[id?,`, `[id?:`, `[id?]`,
    /// `[modifier id`, and `[]`.
    fn next_is_unambiguously_index_signature(&mut self) -> bool {
        self.next_token();
        if matches!(
            self.token,
            SyntaxKind::DotDotDotToken | SyntaxKind::CloseBracketToken
        ) {
            return true;
        }
        if self.token.is_modifier_kind() {
            self.next_token();
            if self.is_identifier() {
                return true;
            }
        } else if !self.is_identifier() {
            return false;
        } else {
            self.next_token();
        }
        if matches!(self.token, SyntaxKind::ColonToken | SyntaxKind::CommaToken) {
            return true;
        }
        // After `?`, only these tokens rule out a conditional expression in a computed name.
        if self.token != SyntaxKind::QuestionToken {
            return false;
        }
        self.next_token();
        matches!(
            self.token,
            SyntaxKind::ColonToken | SyntaxKind::CommaToken | SyntaxKind::CloseBracketToken
        )
    }

    pub(super) fn parse_index_signature_declaration(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
    ) -> NodeId {
        let parameters = self
            .parse_bracketed_list(
                ParsingContext::Parameters,
                |parser| Some(parser.parse_parameter()),
                SyntaxKind::OpenBracketToken,
                SyntaxKind::CloseBracketToken,
            )
            .expect("index signature parameters always parse");
        let type_node = self.parse_type_annotation();
        self.parse_type_member_semicolon();
        let result = self.finish_node(
            SyntaxKind::IndexSignature,
            pos,
            NodeData::IndexSignatureDeclaration(IndexSignatureDeclaration {
                modifiers,
                parameters,
                type_node,
            }),
        );
        self.with_jsdoc(result, jsdoc);
        result
    }

    fn parse_property_or_method_signature(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
    ) -> NodeId {
        let name = self.parse_property_name();
        let postfix_token = self.parse_optional_token(SyntaxKind::QuestionToken);
        let (kind, data) = if matches!(
            self.token,
            SyntaxKind::OpenParenToken | SyntaxKind::LessThanToken
        ) {
            // Method signatures have neither [Yield] nor [Await].
            let type_parameters = self.parse_type_parameters();
            let parameters = self.parse_parameters(SignatureFlags::TYPE);
            let type_node = self.parse_return_type(SyntaxKind::ColonToken, true);
            (
                SyntaxKind::MethodSignature,
                NodeData::MethodSignatureDeclaration(MethodSignatureDeclaration {
                    modifiers,
                    name,
                    postfix_token,
                    type_parameters,
                    parameters,
                    type_node,
                }),
            )
        } else {
            let type_node = self.parse_type_annotation();
            // An initializer is parsed so the checker can report that it is not allowed.
            let initializer = if self.token == SyntaxKind::EqualsToken {
                self.parse_initializer()
            } else {
                None
            };
            (
                SyntaxKind::PropertySignature,
                NodeData::PropertySignatureDeclaration(PropertySignatureDeclaration {
                    modifiers,
                    name,
                    postfix_token,
                    type_node,
                    initializer,
                }),
            )
        };
        self.parse_type_member_semicolon();
        let result = self.finish_node(kind, pos, data);
        self.with_jsdoc(result, jsdoc);
        result
    }
}
