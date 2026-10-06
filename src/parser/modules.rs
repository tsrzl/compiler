//! Namespaces and modules, imports, and exports.

use crate::ast::{
    ExportAssignment, ExportDeclaration, ExportSpecifier, ExternalModuleReference, Identifier,
    ImportClause, ImportDeclaration, ImportEqualsDeclaration, ImportSpecifier, ModifierList,
    ModuleBlock, ModuleDeclaration, NamedExports, NamedImports, NamespaceExport,
    NamespaceExportDeclaration, NamespaceImport, NodeData, NodeFlags, NodeId, SyntaxKind,
};
use crate::diagnostics;

use super::{JsdocScannerInfo, Parser, ParsingContext};

/// The parts of an import or export specifier.
struct Specifier {
    is_type_only: bool,
    property_name: Option<NodeId>,
    name: NodeId,
}

impl Parser<'_> {
    pub(super) fn parse_module_declaration(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
    ) -> NodeId {
        if self.token == SyntaxKind::GlobalKeyword {
            // A global augmentation.
            return self.parse_ambient_external_module_declaration(pos, jsdoc, modifiers);
        }
        let keyword = if self.parse_optional(SyntaxKind::NamespaceKeyword) {
            SyntaxKind::NamespaceKeyword
        } else {
            self.parse_expected(SyntaxKind::ModuleKeyword);
            if self.token == SyntaxKind::StringLiteral {
                return self.parse_ambient_external_module_declaration(pos, jsdoc, modifiers);
            }
            SyntaxKind::ModuleKeyword
        };
        self.parse_module_or_namespace_declaration(pos, Some(jsdoc), modifiers, false, keyword)
    }

    fn parse_ambient_external_module_declaration(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
    ) -> NodeId {
        let saved_has_await_identifier = self.statement_has_await_identifier;
        let (keyword, name) = if self.token == SyntaxKind::GlobalKeyword {
            (SyntaxKind::GlobalKeyword, self.parse_identifier())
        } else {
            (SyntaxKind::ModuleKeyword, self.parse_literal_expression())
        };
        let body = if self.token == SyntaxKind::OpenBraceToken {
            Some(self.parse_module_block())
        } else {
            self.parse_semicolon();
            None
        };
        let result = self.finish_module_declaration(pos, modifiers, keyword, name, body);
        self.with_jsdoc(result, jsdoc);
        self.statement_has_await_identifier = saved_has_await_identifier;
        result
    }

    fn parse_module_block(&mut self) -> NodeId {
        let pos = self.node_pos();
        let statements = if self.parse_expected(SyntaxKind::OpenBraceToken) {
            let statements =
                self.parse_list(ParsingContext::BlockStatements, Self::parse_statement);
            self.parse_expected(SyntaxKind::CloseBraceToken);
            statements
        } else {
            let pos = self.node_pos();
            self.builder.add_missing_list(super::to_u32(pos))
        };
        self.finish_node(
            SyntaxKind::ModuleBlock,
            pos,
            NodeData::ModuleBlock(ModuleBlock { statements }),
        )
    }

    /// Parses `namespace A.B.C { }`, where each dotted name nests an implicitly exported module.
    fn parse_module_or_namespace_declaration(
        &mut self,
        pos: usize,
        jsdoc: Option<JsdocScannerInfo>,
        modifiers: Option<ModifierList>,
        nested: bool,
        keyword: SyntaxKind,
    ) -> NodeId {
        let saved_has_await_identifier = self.statement_has_await_identifier;
        let name = if nested {
            self.parse_identifier_name()
        } else {
            self.parse_identifier()
        };
        let body = if self.parse_optional(SyntaxKind::DotToken) {
            let implicit_pos = self.node_pos();
            let offset = super::to_u32(implicit_pos);
            let implicit_export = self.builder.add_node(
                SyntaxKind::ExportKeyword,
                offset,
                offset,
                NodeFlags::REPARSED,
                NodeData::Token,
            );
            let implicit_modifiers =
                self.new_modifier_list(implicit_pos, implicit_pos, vec![implicit_export]);
            self.parse_module_or_namespace_declaration(
                implicit_pos,
                None,
                Some(implicit_modifiers),
                true,
                keyword,
            )
        } else {
            self.parse_module_block()
        };
        let result = self.finish_module_declaration(pos, modifiers, keyword, name, Some(body));
        if let Some(jsdoc) = jsdoc {
            self.with_jsdoc(result, jsdoc);
        }
        self.statement_has_await_identifier = saved_has_await_identifier;
        result
    }

    fn finish_module_declaration(
        &mut self,
        pos: usize,
        modifiers: Option<ModifierList>,
        keyword: SyntaxKind,
        name: NodeId,
        body: Option<NodeId>,
    ) -> NodeId {
        self.finish_node(
            SyntaxKind::ModuleDeclaration,
            pos,
            NodeData::ModuleDeclaration(ModuleDeclaration {
                modifiers,
                keyword,
                name,
                body,
            }),
        )
    }

    pub(super) fn parse_import_declaration_or_import_equals_declaration(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
    ) -> NodeId {
        self.parse_expected(SyntaxKind::ImportKeyword);
        let after_import_pos = self.node_pos();
        // The identifier is not parsed in an await context; the checker reports that error.
        let saved_has_await_identifier = self.statement_has_await_identifier;
        let mut identifier = self.is_identifier().then(|| self.parse_identifier());
        let mut phase_modifier = None;
        match identifier
            .map(|identifier| self.identifier_text(identifier).to_owned())
            .as_deref()
        {
            Some("type") if self.is_type_only_import_phase() => {
                phase_modifier = Some(SyntaxKind::TypeKeyword);
                identifier = self.is_identifier().then(|| self.parse_identifier());
            }
            Some("defer") if self.is_defer_import_phase() => {
                phase_modifier = Some(SyntaxKind::DeferKeyword);
                identifier = self.is_identifier().then(|| self.parse_identifier());
            }
            _ => {}
        }
        if let Some(identifier) = identifier
            && !matches!(self.token, SyntaxKind::CommaToken | SyntaxKind::FromKeyword)
            && phase_modifier != Some(SyntaxKind::DeferKeyword)
        {
            let is_type_only = phase_modifier == Some(SyntaxKind::TypeKeyword);
            let result = self.parse_import_equals_declaration(
                pos,
                jsdoc,
                modifiers,
                identifier,
                is_type_only,
            );
            // Import-equals declarations are always parsed in an [Await] context.
            self.statement_has_await_identifier = saved_has_await_identifier;
            return result;
        }
        let import_clause =
            self.try_parse_import_clause(identifier, after_import_pos, phase_modifier);
        // Import clauses are always parsed in an [Await] context.
        self.statement_has_await_identifier = saved_has_await_identifier;
        let module_specifier = self.parse_module_specifier();
        let attributes = self.try_parse_import_attributes();
        self.parse_semicolon();
        let result = self.finish_node(
            SyntaxKind::ImportDeclaration,
            pos,
            NodeData::ImportDeclaration(ImportDeclaration {
                modifiers,
                import_clause,
                module_specifier,
                attributes,
            }),
        );
        self.with_jsdoc(result, jsdoc);
        result
    }

    /// After `import type`, decides whether `type` is the type-only modifier or the binding.
    fn is_type_only_import_phase(&mut self) -> bool {
        (self.token != SyntaxKind::FromKeyword
            || (self.is_identifier()
                && self.look_ahead(|parser| {
                    parser.next_token();
                    matches!(
                        parser.token,
                        SyntaxKind::FromKeyword | SyntaxKind::EqualsToken
                    )
                })))
            && (self.is_identifier()
                || matches!(
                    self.token,
                    SyntaxKind::AsteriskToken | SyntaxKind::OpenBraceToken
                ))
    }

    /// After `import defer`, decides whether `defer` is the phase modifier or the binding.
    fn is_defer_import_phase(&mut self) -> bool {
        if self.token == SyntaxKind::FromKeyword {
            !self.look_ahead(Self::next_token_is_token_string_literal)
        } else {
            !matches!(self.token, SyntaxKind::CommaToken | SyntaxKind::EqualsToken)
        }
    }

    fn parse_import_equals_declaration(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
        name: NodeId,
        is_type_only: bool,
    ) -> NodeId {
        self.parse_expected(SyntaxKind::EqualsToken);
        let module_reference = self.parse_module_reference();
        self.parse_semicolon();
        let result = self.finish_node(
            SyntaxKind::ImportEqualsDeclaration,
            pos,
            NodeData::ImportEqualsDeclaration(ImportEqualsDeclaration {
                modifiers,
                is_type_only,
                name,
                module_reference,
            }),
        );
        self.with_jsdoc(result, jsdoc);
        result
    }

    fn parse_module_reference(&mut self) -> NodeId {
        if self.token == SyntaxKind::RequireKeyword
            && self.look_ahead(|parser| parser.next_token() == SyntaxKind::OpenParenToken)
        {
            return self.parse_external_module_reference();
        }
        self.parse_entity_name(false, None)
    }

    fn parse_external_module_reference(&mut self) -> NodeId {
        let saved_has_await_identifier = self.statement_has_await_identifier;
        let pos = self.node_pos();
        self.parse_expected(SyntaxKind::RequireKeyword);
        self.parse_expected(SyntaxKind::OpenParenToken);
        let expression = self.parse_module_specifier();
        self.parse_expected(SyntaxKind::CloseParenToken);
        let result = self.finish_node(
            SyntaxKind::ExternalModuleReference,
            pos,
            NodeData::ExternalModuleReference(ExternalModuleReference { expression }),
        );
        self.statement_has_await_identifier = saved_has_await_identifier;
        result
    }

    /// Parses a module specifier. Any expression is accepted here; the grammar checker reports
    /// anything but a string literal.
    fn parse_module_specifier(&mut self) -> NodeId {
        if self.token == SyntaxKind::StringLiteral {
            return self.parse_literal_expression();
        }
        self.parse_expression()
    }

    fn try_parse_import_clause(
        &mut self,
        identifier: Option<NodeId>,
        pos: usize,
        phase_modifier: Option<SyntaxKind>,
    ) -> Option<NodeId> {
        if identifier.is_none()
            && !matches!(
                self.token,
                SyntaxKind::AsteriskToken | SyntaxKind::OpenBraceToken
            )
        {
            return None;
        }
        let import_clause = self.parse_import_clause(identifier, pos, phase_modifier);
        self.parse_expected(SyntaxKind::FromKeyword);
        Some(import_clause)
    }

    fn parse_import_clause(
        &mut self,
        name: Option<NodeId>,
        pos: usize,
        phase_modifier: Option<SyntaxKind>,
    ) -> NodeId {
        let saved_has_await_identifier = self.statement_has_await_identifier;
        // Namespace or named imports follow when there is no default binding or after a comma.
        let named_bindings =
            (name.is_none() || self.parse_optional(SyntaxKind::CommaToken)).then(|| {
                if self.token == SyntaxKind::AsteriskToken {
                    self.parse_namespace_import()
                } else {
                    self.parse_named_imports()
                }
            });
        let result = self.finish_node(
            SyntaxKind::ImportClause,
            pos,
            NodeData::ImportClause(ImportClause {
                phase_modifier,
                name,
                named_bindings,
            }),
        );
        self.statement_has_await_identifier = saved_has_await_identifier;
        result
    }

    fn parse_namespace_import(&mut self) -> NodeId {
        let pos = self.node_pos();
        self.parse_expected(SyntaxKind::AsteriskToken);
        self.parse_expected(SyntaxKind::AsKeyword);
        let name = self.parse_identifier();
        self.finish_node(
            SyntaxKind::NamespaceImport,
            pos,
            NodeData::NamespaceImport(NamespaceImport { name }),
        )
    }

    fn parse_named_imports(&mut self) -> NodeId {
        let pos = self.node_pos();
        let elements = self
            .parse_bracketed_list(
                ParsingContext::ImportOrExportSpecifiers,
                |parser| Some(parser.parse_import_specifier()),
                SyntaxKind::OpenBraceToken,
                SyntaxKind::CloseBraceToken,
            )
            .expect("import specifiers always parse");
        self.finish_node(
            SyntaxKind::NamedImports,
            pos,
            NodeData::NamedImports(NamedImports { elements }),
        )
    }

    fn parse_import_specifier(&mut self) -> NodeId {
        let pos = self.node_pos();
        let specifier = self.parse_import_or_export_specifier(SyntaxKind::ImportSpecifier);
        // An import binding must be an identifier; a string name is replaced by a missing one.
        let name = if self.builder.node(specifier.name).kind() == SyntaxKind::Identifier {
            specifier.name
        } else {
            let node = self.builder.node(specifier.name);
            let (name_pos, end) = (node.pos() as usize, node.end() as usize);
            let start = self.skip_trivia(name_pos);
            self.parse_error_at(start, end, diagnostics::IDENTIFIER_EXPECTED, &[]);
            self.finish_node(
                SyntaxKind::Identifier,
                name_pos,
                NodeData::Identifier(Identifier { text: "".into() }),
            )
        };
        self.finish_node(
            SyntaxKind::ImportSpecifier,
            pos,
            NodeData::ImportSpecifier(ImportSpecifier {
                is_type_only: specifier.is_type_only,
                property_name: specifier.property_name,
                name,
            }),
        )
    }

    /// Parses an import or export specifier, resolving the many readings of a leading `type`:
    /// `{ type }`, `{ type as }`, `{ type as as }`, and `{ type as as as }` differ in which
    /// identifiers are the property name, the name, and the type-only modifier.
    fn parse_import_or_export_specifier(&mut self, kind: SyntaxKind) -> Specifier {
        let mut can_parse_as_keyword = true;
        let disallow_keywords = kind == SyntaxKind::ImportSpecifier;
        let mut is_type_only = false;
        let mut property_name = None;
        let (mut name, mut name_ok) = self.parse_module_export_name(disallow_keywords);
        if self.builder.node(name).kind() == SyntaxKind::Identifier
            && self.identifier_text(name) == "type"
        {
            if self.token == SyntaxKind::AsKeyword {
                let first_as = self.parse_identifier_name();
                if self.token == SyntaxKind::AsKeyword {
                    let second_as = self.parse_identifier_name();
                    if self.can_parse_module_export_name() {
                        // `{ type as as something }`
                        is_type_only = true;
                        property_name = Some(first_as);
                        (name, name_ok) = self.parse_module_export_name(disallow_keywords);
                    } else {
                        // `{ type as as }`
                        property_name = Some(name);
                        name = second_as;
                    }
                    can_parse_as_keyword = false;
                } else if self.can_parse_module_export_name() {
                    // `{ type as something }`
                    property_name = Some(name);
                    can_parse_as_keyword = false;
                    (name, name_ok) = self.parse_module_export_name(disallow_keywords);
                } else {
                    // `{ type as }`
                    is_type_only = true;
                    name = first_as;
                }
            } else if self.can_parse_module_export_name() {
                // `{ type something }`
                is_type_only = true;
                (name, name_ok) = self.parse_module_export_name(disallow_keywords);
            }
        }
        if can_parse_as_keyword && self.token == SyntaxKind::AsKeyword {
            property_name = Some(name);
            self.parse_expected(SyntaxKind::AsKeyword);
            (name, name_ok) = self.parse_module_export_name(disallow_keywords);
        }
        if !name_ok {
            let node = self.builder.node(name);
            let (start, end) = (self.skip_trivia(node.pos() as usize), node.end() as usize);
            self.parse_error_at(start, end, diagnostics::IDENTIFIER_EXPECTED, &[]);
        }
        Specifier {
            is_type_only,
            property_name,
            name,
        }
    }

    fn can_parse_module_export_name(&self) -> bool {
        self.token.is_identifier_or_keyword() || self.token == SyntaxKind::StringLiteral
    }

    /// Parses a module export name, returning whether it is acceptable where keywords are not.
    fn parse_module_export_name(&mut self, disallow_keywords: bool) -> (NodeId, bool) {
        if self.token == SyntaxKind::StringLiteral {
            return (self.parse_literal_expression(), true);
        }
        let name_ok = !(disallow_keywords && self.token.is_keyword_kind() && !self.is_identifier());
        (self.parse_identifier_name(), name_ok)
    }

    fn try_parse_import_attributes(&mut self) -> Option<NodeId> {
        let is_attributes = self.token == SyntaxKind::WithKeyword
            || (self.token == SyntaxKind::AssertKeyword && !self.has_preceding_line_break());
        if !is_attributes {
            return None;
        }
        if self.token == SyntaxKind::AssertKeyword {
            self.parse_error_at_current_token(
                diagnostics::IMPORT_ASSERTIONS_HAVE_BEEN_REPLACED_BY_IMPORT_ATTRIBUTES_USE_WITH_INSTEAD_OF_ASSERT,
                &[],
            );
        }
        let token = self.token;
        Some(self.parse_import_attributes(token, false))
    }

    pub(super) fn parse_export_assignment(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
    ) -> NodeId {
        let saved_context_flags = self.context_flags;
        let saved_has_await_identifier = self.statement_has_await_identifier;
        self.set_context_flags(NodeFlags::AWAIT_CONTEXT, true);
        let is_export_equals = self.parse_optional(SyntaxKind::EqualsToken);
        if !is_export_equals {
            self.parse_expected(SyntaxKind::DefaultKeyword);
        }
        let expression = self.parse_assignment_expression_or_higher();
        self.parse_semicolon();
        self.context_flags = saved_context_flags;
        self.statement_has_await_identifier = saved_has_await_identifier;
        let result = self.finish_node(
            SyntaxKind::ExportAssignment,
            pos,
            NodeData::ExportAssignment(ExportAssignment {
                modifiers,
                is_export_equals,
                type_node: None,
                expression,
            }),
        );
        self.with_jsdoc(result, jsdoc);
        result
    }

    pub(super) fn parse_namespace_export_declaration(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
    ) -> NodeId {
        self.parse_expected(SyntaxKind::AsKeyword);
        self.parse_expected(SyntaxKind::NamespaceKeyword);
        let saved_has_await_identifier = self.statement_has_await_identifier;
        let name = self.parse_identifier();
        self.statement_has_await_identifier = saved_has_await_identifier;
        self.parse_semicolon();
        // Modifiers are kept so that the grammar checker can report them.
        let result = self.finish_node(
            SyntaxKind::NamespaceExportDeclaration,
            pos,
            NodeData::NamespaceExportDeclaration(NamespaceExportDeclaration { modifiers, name }),
        );
        self.with_jsdoc(result, jsdoc);
        result
    }

    pub(super) fn parse_export_declaration(
        &mut self,
        pos: usize,
        jsdoc: JsdocScannerInfo,
        modifiers: Option<ModifierList>,
    ) -> NodeId {
        let saved_context_flags = self.context_flags;
        let saved_has_await_identifier = self.statement_has_await_identifier;
        self.set_context_flags(NodeFlags::AWAIT_CONTEXT, true);
        let is_type_only = self.parse_optional(SyntaxKind::TypeKeyword);
        let namespace_export_pos = self.node_pos();
        let mut export_clause = None;
        let mut module_specifier = None;
        if self.parse_optional(SyntaxKind::AsteriskToken) {
            if self.parse_optional(SyntaxKind::AsKeyword) {
                export_clause = Some(self.parse_namespace_export(namespace_export_pos));
            }
            self.parse_expected(SyntaxKind::FromKeyword);
            module_specifier = Some(self.parse_module_specifier());
        } else {
            export_clause = Some(self.parse_named_exports());
            // A missing `from` is common; a string on the same line still reads as the specifier.
            if self.token == SyntaxKind::FromKeyword
                || (self.token == SyntaxKind::StringLiteral && !self.has_preceding_line_break())
            {
                self.parse_expected(SyntaxKind::FromKeyword);
                module_specifier = Some(self.parse_module_specifier());
            }
        }
        let attributes = if module_specifier.is_some()
            && matches!(
                self.token,
                SyntaxKind::WithKeyword | SyntaxKind::AssertKeyword
            )
            && !self.has_preceding_line_break()
        {
            if self.token == SyntaxKind::AssertKeyword {
                self.parse_error_at_current_token(
                    diagnostics::IMPORT_ASSERTIONS_HAVE_BEEN_REPLACED_BY_IMPORT_ATTRIBUTES_USE_WITH_INSTEAD_OF_ASSERT,
                    &[],
                );
            }
            let token = self.token;
            Some(self.parse_import_attributes(token, false))
        } else {
            None
        };
        self.parse_semicolon();
        self.context_flags = saved_context_flags;
        self.statement_has_await_identifier = saved_has_await_identifier;
        let result = self.finish_node(
            SyntaxKind::ExportDeclaration,
            pos,
            NodeData::ExportDeclaration(ExportDeclaration {
                modifiers,
                is_type_only,
                export_clause,
                module_specifier,
                attributes,
            }),
        );
        self.with_jsdoc(result, jsdoc);
        result
    }

    fn parse_namespace_export(&mut self, pos: usize) -> NodeId {
        let (name, _) = self.parse_module_export_name(false);
        self.finish_node(
            SyntaxKind::NamespaceExport,
            pos,
            NodeData::NamespaceExport(NamespaceExport { name }),
        )
    }

    fn parse_named_exports(&mut self) -> NodeId {
        let pos = self.node_pos();
        let elements = self
            .parse_bracketed_list(
                ParsingContext::ImportOrExportSpecifiers,
                |parser| Some(parser.parse_export_specifier()),
                SyntaxKind::OpenBraceToken,
                SyntaxKind::CloseBraceToken,
            )
            .expect("export specifiers always parse");
        self.finish_node(
            SyntaxKind::NamedExports,
            pos,
            NodeData::NamedExports(NamedExports { elements }),
        )
    }

    fn parse_export_specifier(&mut self) -> NodeId {
        let pos = self.node_pos();
        let jsdoc = self.jsdoc_scanner_info();
        let specifier = self.parse_import_or_export_specifier(SyntaxKind::ExportSpecifier);
        let result = self.finish_node(
            SyntaxKind::ExportSpecifier,
            pos,
            NodeData::ExportSpecifier(ExportSpecifier {
                is_type_only: specifier.is_type_only,
                property_name: specifier.property_name,
                name: specifier.name,
            }),
        );
        self.with_jsdoc(result, jsdoc);
        result
    }
}
