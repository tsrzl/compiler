//! Namespace, ambient module, and UMD global binding, modeled on TypeScript-Go's
//! `bindModuleDeclaration` and `bindNamespaceExportDeclaration`.

use super::PatternAmbientModule;
use super::binder::{Binder, Table};
use crate::ast::{ModifierFlags, ModuleInstanceState, NodeId, SymbolFlags, SyntaxKind};
use crate::diagnostics;

impl Binder<'_> {
    pub(super) fn bind_module_declaration(&mut self, node: NodeId) {
        let ast = self.ast;
        self.set_export_context_flag(node);
        if !ast.is_ambient_module(node) {
            self.bind_namespace(node);
            return;
        }
        if ast.has_syntactic_modifier(node, ModifierFlags::EXPORT) {
            self.error_on_first_token(
                node,
                diagnostics::X_EXPORT_MODIFIER_CANNOT_BE_APPLIED_TO_AMBIENT_MODULES_AND_MODULE_AUGMENTATIONS_SINCE_THEY_ARE_ALWAYS_VISIBLE,
                &[],
            );
        }
        if self.is_module_augmentation_external(node) {
            self.declare_module_symbol(node);
            return;
        }
        let symbol = self.declare_symbol_and_add_to_symbol_table(
            node,
            SymbolFlags::VALUE_MODULE,
            SymbolFlags::VALUE_MODULE_EXCLUDES,
        );
        let name = ast
            .node(node)
            .data()
            .as_module_declaration()
            .expect("a module declaration node carries module data")
            .name;
        if ast.node(name).kind() == SyntaxKind::StringLiteral {
            let pattern = ast.node_text(name).unwrap_or_default();
            let mut stars = pattern.match_indices('*').map(|(index, _)| index);
            match (stars.next(), stars.next()) {
                (_, Some(_)) => self.error_on_first_token(
                    name,
                    diagnostics::PATTERN_0_CAN_HAVE_AT_MOST_ONE_ASTERISK_CHARACTER,
                    &[pattern],
                ),
                (Some(star_index), None) => {
                    self.pattern_ambient_modules.push(PatternAmbientModule {
                        pattern: pattern.to_owned(),
                        star_index,
                        symbol,
                    });
                }
                (None, None) => {}
            }
        }
    }

    /// Binds a non-ambient namespace and tracks whether it contains only const enums.
    fn bind_namespace(&mut self, node: NodeId) {
        let state = self.declare_module_symbol(node);
        if state == ModuleInstanceState::NonInstantiated {
            return;
        }
        let symbol = self
            .symbol_of(node)
            .expect("a declared namespace has a symbol");
        let flags = self.symbols.symbol(symbol).flags;
        // A namespace merged with a function, class, or regular enum, or one that was found not
        // to be const-enum-only in an earlier declaration, is never const-enum-only.
        let const_enum_only = !flags
            .intersects(SymbolFlags::FUNCTION | SymbolFlags::CLASS | SymbolFlags::REGULAR_ENUM)
            && state == ModuleInstanceState::ConstEnumOnly
            && !self.not_const_enum_only_modules.contains(&symbol);
        let declared = self.symbols.symbol_mut(symbol);
        if const_enum_only {
            declared.flags |= SymbolFlags::CONST_ENUM_ONLY_MODULE;
        } else {
            declared.flags = declared.flags.without(SymbolFlags::CONST_ENUM_ONLY_MODULE);
            self.not_const_enum_only_modules.insert(symbol);
        }
    }

    fn declare_module_symbol(&mut self, node: NodeId) -> ModuleInstanceState {
        let state = self.ast.module_instance_state(node);
        let (includes, excludes) = if state == ModuleInstanceState::NonInstantiated {
            (
                SymbolFlags::NAMESPACE_MODULE,
                SymbolFlags::NAMESPACE_MODULE_EXCLUDES,
            )
        } else {
            (
                SymbolFlags::VALUE_MODULE,
                SymbolFlags::VALUE_MODULE_EXCLUDES,
            )
        };
        self.declare_symbol_and_add_to_symbol_table(node, includes, excludes);
        state
    }

    /// Returns whether an ambient module augments an external module: it is at the top level of
    /// a module, or directly inside a top-level ambient module of a script.
    fn is_module_augmentation_external(&self, node: NodeId) -> bool {
        let ast = self.ast;
        let Some(parent) = ast.node(node).parent() else {
            return false;
        };
        match ast.node(parent).kind() {
            SyntaxKind::SourceFile => self.is_external_module(),
            SyntaxKind::ModuleBlock => ast.node(parent).parent().is_some_and(|grandparent| {
                ast.is_ambient_module(grandparent)
                    && ast.node(grandparent).parent() == Some(ast.root())
                    && !self.is_external_module()
            }),
            _ => false,
        }
    }

    pub(super) fn bind_namespace_export_declaration(&mut self, node: NodeId) {
        let ast = self.ast;
        if ast.node(node).data().modifiers().is_some() {
            self.error_on_node(node, diagnostics::MODIFIERS_CANNOT_APPEAR_HERE, &[]);
        }
        if ast.node(node).parent() != Some(ast.root()) {
            self.error_on_node(
                node,
                diagnostics::GLOBAL_MODULE_EXPORTS_MAY_ONLY_APPEAR_AT_TOP_LEVEL,
                &[],
            );
        } else if !self.is_external_module() {
            self.error_on_node(
                node,
                diagnostics::GLOBAL_MODULE_EXPORTS_MAY_ONLY_APPEAR_IN_MODULE_FILES,
                &[],
            );
        } else if !self.is_declaration_file() {
            self.error_on_node(
                node,
                diagnostics::GLOBAL_MODULE_EXPORTS_MAY_ONLY_APPEAR_IN_DECLARATION_FILES,
                &[],
            );
        } else {
            let file = self.symbol_of(ast.root());
            self.declare_symbol(
                Table::GlobalExports,
                file,
                node,
                SymbolFlags::ALIAS,
                SymbolFlags::ALIAS_EXCLUDES,
            );
        }
    }
}
