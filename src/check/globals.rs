//! Global symbol merging, modeled on TypeScript-Go's `initializeChecker` and `mergeSymbol`.

use super::checker::Checker;
use super::program::{CheckDiagnostic, NodeRef};
use super::symbol_store::SymbolRef;
use crate::ast::{NodeData, SymbolFlags, SyntaxKind};
use crate::diagnostics::{self, Message};
use crate::parser::ScriptKind;
use crate::symbols::SymbolTable;

/// A symbol table the checker merges into.
#[derive(Debug, Clone, Copy)]
enum MergeTarget {
    Members(u32),
    Exports(u32),
}

impl Checker<'_> {
    /// Merges every script's top-level declarations into the globals. Ambient module
    /// declarations merge last, since they may depend on other globals.
    pub(super) fn initialize(&mut self) {
        let mut ambient_modules = Vec::new();
        for index in 0..self.files.len() {
            if self.files[index].is_external_module() {
                self.merge_umd_globals(index);
            } else {
                self.merge_script_globals(index, &mut ambient_modules);
            }
        }
        self.add_undefined_to_globals_or_error_on_redeclaration();
        for symbol in ambient_modules {
            self.merge_global_symbol(symbol);
        }
    }

    /// Merges a script's top-level declarations, deferring ambient module declarations.
    fn merge_script_globals(&mut self, index: usize, ambient_modules: &mut Vec<SymbolRef>) {
        let file = &self.files[index];
        let locals = file
            .bound()
            .locals(file.parsed().ast().root())
            .cloned()
            .unwrap_or_default();
        if let Some(global_this) = locals.get("globalThis") {
            let symbol = SymbolRef::Bound {
                file: index,
                symbol: global_this,
            };
            for declaration in self.symbol_declarations(symbol) {
                self.error_on(
                    declaration,
                    diagnostics::DECLARATION_NAME_CONFLICTS_WITH_BUILT_IN_GLOBAL_IDENTIFIER_0,
                    &["globalThis"],
                );
            }
        }
        for (name, symbol) in locals.iter() {
            let symbol = SymbolRef::Bound {
                file: index,
                symbol,
            };
            if self.symbol_flags(symbol).intersects(SymbolFlags::MODULE) && name.starts_with('"') {
                ambient_modules.push(symbol);
            } else {
                self.merge_global_symbol(symbol);
            }
        }
    }

    /// Adds a module's `export as namespace` globals with first-in-wins semantics.
    fn merge_umd_globals(&mut self, index: usize) {
        let globals = self.files[index].bound().global_exports();
        for (name, symbol) in globals.iter() {
            if self.globals.get(name).is_none() {
                self.globals.insert(
                    name,
                    SymbolRef::Bound {
                        file: index,
                        symbol,
                    },
                );
            }
        }
    }

    fn merge_global_symbol(&mut self, symbol: SymbolRef) {
        let name = self.symbol_name(symbol).to_owned();
        let merged = match self.globals.get(&name) {
            Some(global) => self.merge_symbol(global, symbol, false),
            None => self.merged_symbol(symbol),
        };
        self.globals.insert(name, merged);
    }

    fn add_undefined_to_globals_or_error_on_redeclaration(&mut self) {
        if let Some(target) = self.globals.get("undefined") {
            for declaration in self.symbol_declarations(target) {
                if !self.is_type_declaration(declaration) {
                    self.error_on(
                        declaration,
                        diagnostics::DECLARATION_NAME_CONFLICTS_WITH_BUILT_IN_GLOBAL_IDENTIFIER_0,
                        &["undefined"],
                    );
                }
            }
        } else {
            self.globals.insert("undefined", self.special.undefined);
        }
    }

    /// Merges `source` into `target`. A bound target is first cloned into a transient symbol, so
    /// bindings are never mutated; conflicting declarations are reported and `source` is kept.
    pub(super) fn merge_symbol(
        &mut self,
        target: SymbolRef,
        source: SymbolRef,
        unidirectional: bool,
    ) -> SymbolRef {
        let target_flags = self.symbol_flags(target);
        let source_flags = self.symbol_flags(source);
        let mergeable = |flags: SymbolFlags| {
            !flags.intersects(excluded_symbol_flags(source_flags))
                || (source_flags | flags).intersects(SymbolFlags::ASSIGNMENT)
        };
        if !mergeable(target_flags) {
            if target_flags.intersects(SymbolFlags::NAMESPACE_MODULE) {
                if target != self.special.global_this {
                    let name = self.symbol_name(target).to_owned();
                    if let Some(&declaration) = self.symbol_declarations(source).first() {
                        let name_node = self.adjusted_node_for_error(declaration);
                        self.error_on(
                            name_node,
                            diagnostics::CANNOT_AUGMENT_MODULE_0_WITH_VALUE_EXPORTS_BECAUSE_IT_RESOLVES_TO_A_NON_MODULE_ENTITY,
                            &[&name],
                        );
                    }
                }
            } else {
                self.report_merge_symbol_error(target, source);
            }
            return target;
        }
        if source == target {
            return target;
        }
        let target = if target_flags.intersects(SymbolFlags::TRANSIENT) {
            target
        } else {
            let resolved = self.resolve_symbol(target);
            if resolved == self.special.unknown {
                return source;
            }
            if !mergeable(self.symbol_flags(resolved)) {
                self.report_merge_symbol_error(target, source);
                return source;
            }
            let clone = self.symbols.clone_symbol(self.files, resolved);
            self.merged_symbols.insert(resolved, clone);
            clone
        };
        let SymbolRef::Transient(id) = target else {
            unreachable!("merge targets are transient");
        };
        let source_view = self.symbols.view(self.files, source);
        let (source_declarations, source_value_declaration) = (
            source_view.declarations.clone(),
            source_view.value_declaration,
        );
        let (source_members, source_exports) =
            (source_view.members.clone(), source_view.exports.clone());
        let transient = self.symbols.transient_mut(id);
        // Merging an instantiated module into a const-enum-only module makes it instantiated.
        if source_flags.intersects(SymbolFlags::VALUE_MODULE)
            && transient.flags.intersects(SymbolFlags::VALUE_MODULE)
            && transient
                .flags
                .intersects(SymbolFlags::CONST_ENUM_ONLY_MODULE)
            && !source_flags.intersects(SymbolFlags::CONST_ENUM_ONLY_MODULE)
        {
            transient.flags = transient.flags.without(SymbolFlags::CONST_ENUM_ONLY_MODULE);
        }
        let mut added_flags = source_flags;
        if !transient
            .flags
            .intersects(SymbolFlags::CONST_ENUM_ONLY_MODULE)
        {
            added_flags = added_flags.without(SymbolFlags::CONST_ENUM_ONLY_MODULE);
        }
        transient.flags |= added_flags;
        transient.declarations.extend(source_declarations);
        if let Some(value_declaration) = source_value_declaration {
            self.set_value_declaration(id, value_declaration);
        }
        self.merge_symbol_table(
            MergeTarget::Members(id),
            &source_members,
            unidirectional,
            None,
        );
        self.merge_symbol_table(
            MergeTarget::Exports(id),
            &source_exports,
            unidirectional,
            Some(target),
        );
        if !unidirectional {
            self.merged_symbols.insert(source, target);
        }
        target
    }

    fn merge_symbol_table(
        &mut self,
        target: MergeTarget,
        source: &SymbolTable<SymbolRef>,
        unidirectional: bool,
        merged_parent: Option<SymbolRef>,
    ) {
        for (name, source_symbol) in source.iter() {
            let target_symbol = self.merge_target_table(target).get(name);
            let merged = match target_symbol {
                Some(target_symbol) => {
                    self.merge_symbol(target_symbol, source_symbol, unidirectional)
                }
                None => self.merged_symbol(source_symbol),
            };
            // A symbol that received a merge belongs to the merged parent that initiated it.
            if let (Some(parent), Some(_), SymbolRef::Transient(merged_id)) =
                (merged_parent, target_symbol, merged)
            {
                self.symbols.transient_mut(merged_id).parent = Some(parent);
            }
            self.merge_target_table_mut(target).insert(name, merged);
        }
    }

    fn merge_target_table(&self, target: MergeTarget) -> &SymbolTable<SymbolRef> {
        match target {
            MergeTarget::Members(id) => &self.symbols.transient(id).members,
            MergeTarget::Exports(id) => &self.symbols.transient(id).exports,
        }
    }

    fn merge_target_table_mut(&mut self, target: MergeTarget) -> &mut SymbolTable<SymbolRef> {
        match target {
            MergeTarget::Members(id) => &mut self.symbols.transient_mut(id).members,
            MergeTarget::Exports(id) => &mut self.symbols.transient_mut(id).exports,
        }
    }

    /// Returns the symbol an alias refers to. Alias resolution is ported with name resolution;
    /// until then every symbol resolves to itself.
    #[expect(
        clippy::unused_self,
        reason = "alias resolution reads the checker once ported"
    )]
    pub(super) const fn resolve_symbol(&self, symbol: SymbolRef) -> SymbolRef {
        symbol
    }

    /// Records the value declaration of a merged symbol: non-assignment declarations win over
    /// assignment declarations, and non-namespace declarations win over namespaces.
    fn set_value_declaration(&mut self, id: u32, node: NodeRef) {
        let kind_of = |checker: &Self, node: NodeRef| {
            checker.files[node.file]
                .parsed()
                .ast()
                .node(node.node)
                .kind()
        };
        let is_assignment_declaration = |kind: SyntaxKind| {
            matches!(
                kind,
                SyntaxKind::BinaryExpression
                    | SyntaxKind::PropertyAccessExpression
                    | SyntaxKind::ElementAccessExpression
                    | SyntaxKind::Identifier
                    | SyntaxKind::CallExpression
            )
        };
        let node_kind = kind_of(self, node);
        let replace = self
            .symbols
            .transient(id)
            .value_declaration
            .is_none_or(|current| {
                let current_kind = kind_of(self, current);
                is_assignment_declaration(current_kind) && !is_assignment_declaration(node_kind)
                    || current_kind != node_kind
                        && matches!(
                            current_kind,
                            SyntaxKind::ModuleDeclaration | SyntaxKind::Identifier
                        )
            });
        if replace {
            self.symbols.transient_mut(id).value_declaration = Some(node);
        }
    }

    fn report_merge_symbol_error(&mut self, target: SymbolRef, source: SymbolRef) {
        let either = self.symbol_flags(target) | self.symbol_flags(source);
        let message = if either.intersects(SymbolFlags::ENUM) {
            diagnostics::ENUM_DECLARATIONS_CAN_ONLY_MERGE_WITH_NAMESPACE_OR_OTHER_ENUM_DECLARATIONS
        } else if either.intersects(SymbolFlags::BLOCK_SCOPED_VARIABLE) {
            diagnostics::CANNOT_REDECLARE_BLOCK_SCOPED_VARIABLE_0
        } else {
            diagnostics::DUPLICATE_IDENTIFIER_0
        };
        let name = self.symbol_name(source).to_owned();
        let source_declarations = self.symbol_declarations(source);
        let target_declarations = self.symbol_declarations(target);
        // Plain JavaScript files do not report duplicate declarations.
        if !self.is_plain_js_symbol(&source_declarations) {
            for &declaration in &source_declarations {
                self.add_duplicate_declaration_error(
                    declaration,
                    message,
                    &name,
                    &target_declarations,
                );
            }
        }
        if !self.is_plain_js_symbol(&target_declarations) {
            for &declaration in &target_declarations {
                self.add_duplicate_declaration_error(
                    declaration,
                    message,
                    &name,
                    &source_declarations,
                );
            }
        }
    }

    fn is_plain_js_symbol(&self, declarations: &[NodeRef]) -> bool {
        declarations.first().is_some_and(|declaration| {
            matches!(
                self.files[declaration.file]
                    .parsed()
                    .options()
                    .script_kind(),
                ScriptKind::Js | ScriptKind::Jsx
            )
        })
    }

    /// Reports a duplicate declaration once, relating up to five other declarations.
    fn add_duplicate_declaration_error(
        &mut self,
        node: NodeRef,
        message: Message,
        name: &str,
        related_nodes: &[NodeRef],
    ) {
        let error_node = self.adjusted_node_for_error(node);
        let diagnostic = self.diagnostic_for(error_node, message, &[name]);
        let index = self.lookup_or_issue_error(diagnostic);
        for &related in related_nodes {
            let adjusted = self.adjusted_node_for_error(related);
            if adjusted == error_node {
                continue;
            }
            let leading =
                self.diagnostic_for(adjusted, diagnostics::X_0_WAS_ALSO_DECLARED_HERE, &[name]);
            let follow_on = self.diagnostic_for(adjusted, diagnostics::X_AND_HERE, &[]);
            let error = self.diagnostic_mut(index);
            if error.related().len() >= 5
                || error.related().iter().any(|existing| {
                    existing.same_report(&follow_on) || existing.same_report(&leading)
                })
            {
                continue;
            }
            error.add_related(if error.related().is_empty() {
                leading
            } else {
                follow_on
            });
        }
    }

    fn adjusted_node_for_error(&self, node: NodeRef) -> NodeRef {
        let ast = self.files[node.file].parsed().ast();
        NodeRef {
            file: node.file,
            node: ast.name_of_declaration(node.node).unwrap_or(node.node),
        }
    }

    fn is_type_declaration(&self, node: NodeRef) -> bool {
        let ast = self.files[node.file].parsed().ast();
        let data = ast.node(node.node).data();
        match ast.node(node.node).kind() {
            SyntaxKind::TypeParameter
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::EnumDeclaration => true,
            SyntaxKind::ImportClause => is_type_only(data),
            SyntaxKind::ImportSpecifier | SyntaxKind::ExportSpecifier => ast
                .node(node.node)
                .parent()
                .and_then(|list| ast.node(list).parent())
                .is_some_and(|owner| is_type_only(ast.node(owner).data())),
            _ => false,
        }
    }

    pub(super) fn diagnostic_for(
        &self,
        node: NodeRef,
        message: Message,
        arguments: &[&str],
    ) -> CheckDiagnostic {
        CheckDiagnostic::new(
            node.file,
            self.files[node.file].diagnostic_for_node(node.node, message, arguments),
        )
    }

    pub(super) fn error_on(&mut self, node: NodeRef, message: Message, arguments: &[&str]) {
        let diagnostic = self.diagnostic_for(node, message, arguments);
        self.add_diagnostic(diagnostic);
    }
}

fn is_type_only(data: &NodeData) -> bool {
    match data {
        NodeData::ImportClause(clause) => clause.phase_modifier == Some(SyntaxKind::TypeKeyword),
        NodeData::ExportDeclaration(declaration) => declaration.is_type_only,
        _ => false,
    }
}

/// Returns the flags a declaration with `flags` cannot merge with.
fn excluded_symbol_flags(flags: SymbolFlags) -> SymbolFlags {
    const EXCLUSIONS: [(SymbolFlags, SymbolFlags); 16] = [
        (
            SymbolFlags::BLOCK_SCOPED_VARIABLE,
            SymbolFlags::BLOCK_SCOPED_VARIABLE_EXCLUDES,
        ),
        (
            SymbolFlags::FUNCTION_SCOPED_VARIABLE,
            SymbolFlags::FUNCTION_SCOPED_VARIABLE_EXCLUDES,
        ),
        (SymbolFlags::PROPERTY, SymbolFlags::PROPERTY_EXCLUDES),
        (SymbolFlags::ENUM_MEMBER, SymbolFlags::ENUM_MEMBER_EXCLUDES),
        (SymbolFlags::FUNCTION, SymbolFlags::FUNCTION_EXCLUDES),
        (SymbolFlags::CLASS, SymbolFlags::CLASS_EXCLUDES),
        (SymbolFlags::INTERFACE, SymbolFlags::INTERFACE_EXCLUDES),
        (
            SymbolFlags::REGULAR_ENUM,
            SymbolFlags::REGULAR_ENUM_EXCLUDES,
        ),
        (SymbolFlags::CONST_ENUM, SymbolFlags::CONST_ENUM_EXCLUDES),
        (
            SymbolFlags::VALUE_MODULE,
            SymbolFlags::VALUE_MODULE_EXCLUDES,
        ),
        (SymbolFlags::METHOD, SymbolFlags::METHOD_EXCLUDES),
        (
            SymbolFlags::GET_ACCESSOR,
            SymbolFlags::GET_ACCESSOR_EXCLUDES,
        ),
        (
            SymbolFlags::SET_ACCESSOR,
            SymbolFlags::SET_ACCESSOR_EXCLUDES,
        ),
        (
            SymbolFlags::TYPE_PARAMETER,
            SymbolFlags::TYPE_PARAMETER_EXCLUDES,
        ),
        (SymbolFlags::TYPE_ALIAS, SymbolFlags::TYPE_ALIAS_EXCLUDES),
        (SymbolFlags::ALIAS, SymbolFlags::ALIAS_EXCLUDES),
    ];
    let mut excluded = EXCLUSIONS
        .iter()
        .filter(|(flag, _)| flags.intersects(*flag))
        .fold(SymbolFlags::NONE, |excluded, (_, excludes)| {
            excluded | *excludes
        });
    if flags.intersects(SymbolFlags::REPLACEABLE_BY_METHOD) {
        excluded = excluded.without(SymbolFlags::METHOD);
    }
    excluded
}
