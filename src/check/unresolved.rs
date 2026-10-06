//! Reporting names that do not resolve, modeled on TypeScript-Go's `onFailedToResolveSymbol`.
//!
//! A missing name reports, in order of preference, the lib that would declare it, the closest
//! spelling in scope, or a plain "Cannot find name" error. The checks for misuse, such as a type
//! used as a value, are ported with the expressions and type references they apply to.

use std::cmp::Ordering;

use super::checker::Checker;
use super::feature_map::features_of;
use super::name_resolution::LookupMode;
use super::program::NodeRef;
use super::symbol_store::SymbolRef;
use crate::ast::{SymbolFlags, SyntaxKind};
use crate::diagnostics::{self, Message};
use crate::spelling::spelling_suggestion_by;
use crate::symbols::{INTERNAL_SYMBOL_NAME_PREFIX, SymbolTable};

/// Built-in object types whose primitive type is suggested for a near-miss of the primitive name.
pub(super) const PRIMITIVE_TYPE_ALIASES: [(&str, &str); 6] = [
    ("string", "String"),
    ("number", "Number"),
    ("boolean", "Boolean"),
    ("object", "Object"),
    ("bigint", "BigInt"),
    ("symbol", "Symbol"),
];

impl Checker<'_> {
    /// Returns the message for a name that does not resolve, which for well-known names
    /// suggests the lib or type definitions that would declare it.
    #[must_use]
    pub fn cannot_find_name_message(&self, node: NodeRef) -> Message {
        let ast = self.files[node.file].parsed().ast();
        let parent_kind = ast
            .node(node.node)
            .parent()
            .map(|parent| ast.node(parent).kind());
        let wildcard = self.options.uses_wildcard_types;
        match ast.identifier_text(node.node).unwrap_or_default() {
            "document" | "console" => {
                diagnostics::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_CHANGE_YOUR_TARGET_LIBRARY_TRY_CHANGING_THE_LIB_COMPILER_OPTION_TO_INCLUDE_DOM
            }
            "$" if wildcard => diagnostics::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_JQUERY_TRY_NPM_I_SAVE_DEV_TYPES_SLASHJQUERY,
            "$" => diagnostics::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_JQUERY_TRY_NPM_I_SAVE_DEV_TYPES_SLASHJQUERY_AND_THEN_ADD_JQUERY_TO_THE_TYPES_FIELD_IN_YOUR_TSCONFIG,
            "beforeEach" | "describe" | "suite" | "it" | "test" if wildcard => diagnostics::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_A_TEST_RUNNER_TRY_NPM_I_SAVE_DEV_TYPES_SLASHJEST_OR_NPM_I_SAVE_DEV_TYPES_SLASHMOCHA,
            "beforeEach" | "describe" | "suite" | "it" | "test" => diagnostics::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_A_TEST_RUNNER_TRY_NPM_I_SAVE_DEV_TYPES_SLASHJEST_OR_NPM_I_SAVE_DEV_TYPES_SLASHMOCHA_AND_THEN_ADD_JEST_OR_MOCHA_TO_THE_TYPES_FIELD_IN_YOUR_TSCONFIG,
            "process" | "require" | "Buffer" | "module" | "NodeJS" if wildcard => diagnostics::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_NODE_TRY_NPM_I_SAVE_DEV_TYPES_SLASHNODE,
            "process" | "require" | "Buffer" | "module" | "NodeJS" => diagnostics::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_NODE_TRY_NPM_I_SAVE_DEV_TYPES_SLASHNODE_AND_THEN_ADD_NODE_TO_THE_TYPES_FIELD_IN_YOUR_TSCONFIG,
            "Bun" if wildcard => diagnostics::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_BUN_TRY_NPM_I_SAVE_DEV_TYPES_SLASHBUN,
            "Bun" => diagnostics::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_INSTALL_TYPE_DEFINITIONS_FOR_BUN_TRY_NPM_I_SAVE_DEV_TYPES_SLASHBUN_AND_THEN_ADD_BUN_TO_THE_TYPES_FIELD_IN_YOUR_TSCONFIG,
            "Map" | "Set" | "Promise" | "ast.Symbol" | "WeakMap" | "WeakSet" | "Iterator" | "AsyncIterator"
            | "SharedArrayBuffer" | "Atomics" | "AsyncIterable" | "AsyncIterableIterator" | "AsyncGenerator"
            | "AsyncGeneratorFunction" | "BigInt" | "Reflect" | "BigInt64Array" | "BigUint64Array" => {
                diagnostics::CANNOT_FIND_NAME_0_DO_YOU_NEED_TO_CHANGE_YOUR_TARGET_LIBRARY_TRY_CHANGING_THE_LIB_COMPILER_OPTION_TO_1_OR_LATER
            }
            "await" if parent_kind == Some(SyntaxKind::CallExpression) => {
                diagnostics::CANNOT_FIND_NAME_0_DID_YOU_MEAN_TO_WRITE_THIS_IN_AN_ASYNC_FUNCTION
            }
            _ if parent_kind == Some(SyntaxKind::ShorthandPropertyAssignment) => {
                diagnostics::NO_VALUE_EXISTS_IN_SCOPE_FOR_THE_SHORTHAND_PROPERTY_0_EITHER_DECLARE_ONE_OR_PROVIDE_AN_INITIALIZER
            }
            _ => diagnostics::CANNOT_FIND_NAME_0,
        }
    }

    /// Reports a name that did not resolve: with the lib that declares it, the closest spelling
    /// in scope, or `message` alone.
    pub(super) fn on_failed_to_resolve_symbol(
        &mut self,
        location: NodeRef,
        name: &str,
        meaning: SymbolFlags,
        message: Message,
    ) {
        let ast = self.files[location.file].parsed().ast();
        let declaration_name = if ast.identifier_text(location.node) == Some(name) {
            self.files[location.file]
                .source_text_of(location.node)
                .to_owned()
        } else {
            name.to_owned()
        };
        if let Some(&(lib, _)) = features_of(name).and_then(|features| features.first()) {
            self.error_on(location, message, &[&declaration_name, lib]);
            return;
        }
        let suggestion = self
            .resolve_name_for_suggestion(location, name, meaning)
            .filter(|&suggestion| {
                !self
                    .value_declaration(suggestion)
                    .is_some_and(|declaration| {
                        let ast = self.files[declaration.file].parsed().ast();
                        ast.is_ambient_module(declaration.node)
                            && ast.is_global_scope_augmentation(declaration.node)
                    })
            });
        let Some(suggestion) = suggestion else {
            self.error_on(location, message, &[&declaration_name]);
            return;
        };
        let suggestion_name = self.symbol_name(suggestion).to_owned();
        let did_you_mean = if meaning == SymbolFlags::NAMESPACE {
            diagnostics::CANNOT_FIND_NAMESPACE_0_DID_YOU_MEAN_1
        } else {
            diagnostics::CANNOT_FIND_NAME_0_DID_YOU_MEAN_1
        };
        let mut diagnostic = self.diagnostic_for(
            location,
            did_you_mean,
            &[&declaration_name, &suggestion_name],
        );
        if let Some(declaration) = self.value_declaration(suggestion) {
            let related = self.diagnostic_for(
                declaration,
                diagnostics::X_0_IS_DECLARED_HERE,
                &[&suggestion_name],
            );
            diagnostic.add_related(related);
        }
        self.add_diagnostic(diagnostic);
    }

    /// Returns the closest spelling of `name` visible from `location`.
    fn resolve_name_for_suggestion(
        &self,
        location: NodeRef,
        name: &str,
        meaning: SymbolFlags,
    ) -> Option<SymbolRef> {
        self.resolve_name_with_mode(location, name, meaning, LookupMode::Suggestion)
    }

    /// Returns the candidate closest to `name` among those whose flags include `meaning`, ignoring
    /// internal and quoted module names.
    pub(super) fn spelling_suggestion_for_name(
        &self,
        name: &str,
        candidates: impl IntoIterator<Item = SymbolRef>,
        meaning: SymbolFlags,
    ) -> Option<SymbolRef> {
        spelling_suggestion_by(
            name,
            candidates,
            |&candidate| {
                let candidate_name = self.symbol_name(candidate);
                let usable = !candidate_name.is_empty()
                    && !candidate_name.starts_with('"')
                    && !candidate_name.starts_with(INTERNAL_SYMBOL_NAME_PREFIX)
                    && self.symbol_flags(candidate).intersects(meaning);
                if usable {
                    candidate_name.to_owned()
                } else {
                    String::new()
                }
            },
            |&left, &right| self.compare_symbols(left, right),
        )
    }

    /// Returns the primitive type suggestions for the built-in object types the table declares.
    pub(super) fn primitive_type_alias_suggestions(
        &self,
        table: &SymbolTable<SymbolRef>,
    ) -> Vec<SymbolRef> {
        PRIMITIVE_TYPE_ALIASES
            .iter()
            .zip(&self.special.primitive_aliases)
            .filter(|((_, builtin), _)| table.get(builtin).is_some())
            .map(|(_, &alias)| alias)
            .collect()
    }

    /// Orders symbols by their first declaration's program position, then by name, then by
    /// identity, as TypeScript-Go's `compareSymbols` does.
    pub(super) fn compare_symbols(&self, left: SymbolRef, right: SymbolRef) -> Ordering {
        if left == right {
            return Ordering::Equal;
        }
        let left_declarations = self.symbol_declarations(left);
        let right_declarations = self.symbol_declarations(right);
        let by_declaration = match (left_declarations.first(), right_declarations.first()) {
            (Some(&first), Some(&second)) => self.compare_nodes(first, second),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        };
        by_declaration
            .then_with(|| self.symbol_name(left).cmp(self.symbol_name(right)))
            .then_with(|| left.cmp(&right))
    }

    /// Orders nodes by file order in the program, then by position.
    pub(super) fn compare_nodes(&self, left: NodeRef, right: NodeRef) -> Ordering {
        if left.file != right.file {
            return left.file.cmp(&right.file);
        }
        let ast = self.files[left.file].parsed().ast();
        ast.node(left.node).pos().cmp(&ast.node(right.node).pos())
    }
}
