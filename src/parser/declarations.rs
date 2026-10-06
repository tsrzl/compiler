//! Declarations: the declaration dispatcher, variables, and functions.

use crate::ast::{
    FunctionDeclaration, MissingDeclaration, ModifierFlags, ModifierList, NodeData, NodeFlags,
    NodeId, SyntaxKind, VariableDeclaration, VariableDeclarationList, VariableStatement,
};
use crate::diagnostics;

use super::modifiers::ModifierOptions;
use super::signatures::SignatureFlags;
use super::{JsdocScannerInfo, Parser, ParsingContext};

impl Parser<'_> {
    /// Parses modifiers and the declaration they apply to; `declare` makes it ambient.
    pub(super) fn parse_declaration(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        let modifiers = self.parse_modifiers_with(ModifierOptions::DECORATED);
        let is_ambient = modifiers
            .as_ref()
            .is_some_and(|modifiers| self.modifiers_contain(modifiers, SyntaxKind::DeclareKeyword));
        if !is_ambient {
            return self.parse_declaration_worker(pos, jsdoc, modifiers);
        }
        if let Some(modifiers) = &modifiers {
            self.mark_modifiers_ambient(modifiers);
        }
        self.do_in_context(NodeFlags::AMBIENT, true, |parser| {
            parser.parse_declaration_worker(pos, jsdoc, modifiers)
        })
    }

    fn modifiers_contain(&self, modifiers: &ModifierList, kind: SyntaxKind) -> bool {
        self.builder
            .list(*modifiers.list())
            .iter()
            .any(|&modifier| self.builder.node(modifier).kind() == kind)
    }

    fn parse_declaration_worker(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
    ) -> NodeId {
        let token = self.token;
        match token {
            SyntaxKind::VarKeyword
            | SyntaxKind::LetKeyword
            | SyntaxKind::ConstKeyword
            | SyntaxKind::UsingKeyword => {
                return self.parse_variable_statement(pos, jsdoc, modifiers);
            }
            SyntaxKind::AwaitKeyword if self.is_await_using_declaration() => {
                return self.parse_variable_statement(pos, jsdoc, modifiers);
            }
            SyntaxKind::FunctionKeyword => {
                return self.parse_function_declaration(pos, jsdoc, modifiers);
            }
            SyntaxKind::ClassKeyword => {
                return self.parse_class_declaration_or_expression(
                    pos,
                    jsdoc,
                    modifiers,
                    SyntaxKind::ClassDeclaration,
                );
            }
            SyntaxKind::InterfaceKeyword => {
                return self.parse_interface_declaration(pos, jsdoc, modifiers);
            }
            SyntaxKind::TypeKeyword => {
                return self.parse_type_alias_declaration(pos, jsdoc, modifiers);
            }
            SyntaxKind::EnumKeyword => return self.parse_enum_declaration(pos, jsdoc, modifiers),
            SyntaxKind::GlobalKeyword
            | SyntaxKind::ModuleKeyword
            | SyntaxKind::NamespaceKeyword => {
                return self.parse_module_declaration(pos, jsdoc, modifiers);
            }
            SyntaxKind::ImportKeyword => {
                return self
                    .parse_import_declaration_or_import_equals_declaration(pos, jsdoc, modifiers);
            }
            SyntaxKind::ExportKeyword => {
                self.next_token();
                return match self.token {
                    SyntaxKind::DefaultKeyword | SyntaxKind::EqualsToken => {
                        self.parse_export_assignment(pos, jsdoc, modifiers)
                    }
                    SyntaxKind::AsKeyword => {
                        self.parse_namespace_export_declaration(pos, jsdoc, modifiers)
                    }
                    _ => self.parse_export_declaration(pos, jsdoc, modifiers),
                };
            }
            _ => {}
        }
        // Decorators or modifiers promised a declaration that did not follow; recover with an
        // incomplete declaration.
        let modifiers = modifiers.expect("a declaration without a keyword has modifiers");
        let node_pos = self.node_pos();
        self.parse_error_at(node_pos, node_pos, diagnostics::DECLARATION_EXPECTED, &[]);
        self.finish_node(
            SyntaxKind::MissingDeclaration,
            pos,
            NodeData::MissingDeclaration(MissingDeclaration {
                modifiers: Some(modifiers),
            }),
        )
    }

    pub(super) fn parse_variable_statement(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
    ) -> NodeId {
        let declaration_list = self.parse_variable_declaration_list(false);
        self.parse_semicolon();
        let result = self.finish_node(
            SyntaxKind::VariableStatement,
            pos,
            NodeData::VariableStatement(VariableStatement {
                modifiers,
                declaration_list,
            }),
        );
        self.with_jsdoc(result, jsdoc);
        result
    }

    pub(super) fn parse_variable_declaration_list(
        &mut self,
        in_for_statement_initializer: bool,
    ) -> NodeId {
        let pos = self.node_pos();
        let token = self.token;
        let flags = match token {
            SyntaxKind::VarKeyword => NodeFlags::NONE,
            SyntaxKind::LetKeyword => NodeFlags::LET,
            SyntaxKind::ConstKeyword => NodeFlags::CONST,
            SyntaxKind::UsingKeyword => NodeFlags::USING,
            // `await` that does not start `await using` declares without block scoping.
            SyntaxKind::AwaitKeyword => {
                if self.is_await_using_declaration() {
                    self.next_token();
                    NodeFlags::AWAIT_USING
                } else {
                    NodeFlags::NONE
                }
            }
            token => unreachable!("{token:?} does not start a variable declaration list"),
        };
        self.next_token();
        // In `for (let of X)`, `of` is the keyword, leaving an empty declaration list for the
        // checker to report.
        let declarations = if self.token == SyntaxKind::OfKeyword
            && self.look_ahead(|parser| {
                parser.next_token();
                parser.is_identifier() && parser.next_token() == SyntaxKind::CloseParenToken
            }) {
            let pos = self.node_pos();
            self.builder.add_missing_list(super::to_u32(pos))
        } else {
            self.do_in_context(
                NodeFlags::DISALLOW_IN_CONTEXT,
                in_for_statement_initializer,
                |parser| {
                    parser.parse_delimited_list(ParsingContext::VariableDeclarations, |parser| {
                        Some(parser.parse_variable_declaration(!in_for_statement_initializer))
                    })
                },
            )
            .expect("variable declarations always parse")
        };
        self.finish_node_with_flags(
            SyntaxKind::VariableDeclarationList,
            pos,
            flags,
            NodeData::VariableDeclarationList(VariableDeclarationList { declarations }),
        )
    }

    pub(super) fn parse_variable_declaration(&mut self, allow_exclamation: bool) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        let name = self.parse_identifier_or_pattern_with_diagnostic(Some(
            diagnostics::PRIVATE_IDENTIFIERS_ARE_NOT_ALLOWED_IN_VARIABLE_DECLARATIONS,
        ));
        let exclamation_token = (allow_exclamation
            && self.builder.node(name).kind() == SyntaxKind::Identifier
            && self.token == SyntaxKind::ExclamationToken
            && !self.has_preceding_line_break())
        .then(|| self.parse_token_node());
        let type_node = self.parse_type_annotation();
        let initializer = if matches!(self.token, SyntaxKind::InKeyword | SyntaxKind::OfKeyword) {
            None
        } else {
            self.parse_initializer()
        };
        let result = self.finish_node(
            SyntaxKind::VariableDeclaration,
            pos,
            NodeData::VariableDeclaration(VariableDeclaration {
                name,
                exclamation_token,
                type_node,
                initializer,
            }),
        );
        self.with_jsdoc(result, jsdoc);
        result
    }

    pub(super) fn parse_function_declaration(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
    ) -> NodeId {
        let modifier_flags = modifiers.map_or(ModifierFlags::NONE, |modifiers| modifiers.flags());
        self.parse_expected(SyntaxKind::FunctionKeyword);
        let asterisk_token = self.parse_optional_token(SyntaxKind::AsteriskToken);
        // The name is not parsed in an await context; the checker reports that grammar error.
        let name = (!modifier_flags.intersects(ModifierFlags::DEFAULT)
            || self.is_binding_identifier())
        .then(|| self.parse_binding_identifier());
        let flags = SignatureFlags::function(
            asterisk_token.is_some(),
            modifier_flags.intersects(ModifierFlags::ASYNC),
        );
        let type_parameters = self.parse_type_parameters();
        let saved_context_flags = self.context_flags;
        if modifier_flags.intersects(ModifierFlags::EXPORT) {
            self.set_context_flags(NodeFlags::AWAIT_CONTEXT, true);
        }
        let parameters = self.parse_parameters(flags);
        let type_node = self.parse_return_type(SyntaxKind::ColonToken, false);
        let body = self.parse_function_block_or_semicolon(flags, Some(diagnostics::X_OR_EXPECTED));
        self.context_flags = saved_context_flags;
        let result = self.finish_node(
            SyntaxKind::FunctionDeclaration,
            pos,
            NodeData::FunctionDeclaration(FunctionDeclaration {
                modifiers,
                asterisk_token,
                name,
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
