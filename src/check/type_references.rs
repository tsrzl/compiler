//! Named type references, modeled on TypeScript-Go's `getTypeFromTypeReference`.
//!
//! Ported so far: identifier references to non-generic type aliases, with circularity detection.
//! Qualified names, classes, interfaces, enums, type parameters, and generic aliases resolve to
//! the error type until their declared types are ported.

use super::checker::Checker;
use super::program::NodeRef;
use super::symbol_store::SymbolRef;
use super::types::TypeId;
use crate::ast::{NodeData, SymbolFlags, SyntaxKind};
use crate::diagnostics;

/// A symbol property whose computation may be circular.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TypeSystemProperty {
    /// The declared type of a type alias.
    DeclaredType,
}

/// An in-progress type resolution and whether it has been found circular.
#[derive(Debug, Clone, Copy)]
pub(super) struct TypeResolution {
    target: SymbolRef,
    property: TypeSystemProperty,
    result: bool,
}

impl Checker<'_> {
    pub(super) fn type_from_type_reference(&mut self, node: NodeRef) -> TypeId {
        let symbol = self.symbol_from_type_reference(node);
        self.type_reference_type(node, symbol)
    }

    /// Returns the symbol a type reference names, cached per node.
    fn symbol_from_type_reference(&mut self, node: NodeRef) -> SymbolRef {
        if let Some(&symbol) = self.symbol_node_links.get(&node) {
            return symbol;
        }
        let symbol = self.resolve_type_reference_name(node, SymbolFlags::TYPE);
        self.symbol_node_links.insert(node, symbol);
        symbol
    }

    /// Resolves a type reference's name, reporting a name that does not resolve.
    fn resolve_type_reference_name(&mut self, node: NodeRef, meaning: SymbolFlags) -> SymbolRef {
        let ast = self.files[node.file].parsed().ast();
        let Some(reference) = ast.node(node.node).data().as_type_reference_node() else {
            return self.special.unknown;
        };
        let name = NodeRef {
            file: node.file,
            node: reference.type_name,
        };
        if ast.node(name.node).kind() != SyntaxKind::Identifier || ast.node_is_missing(name.node) {
            return self.special.unknown;
        }
        let text = ast
            .identifier_text(name.node)
            .unwrap_or_default()
            .to_owned();
        let message = self.cannot_find_name_message(name);
        self.resolve_name_reporting(name, &text, meaning, message)
            .map_or(self.special.unknown, |symbol| self.merged_symbol(symbol))
    }

    fn type_reference_type(&mut self, node: NodeRef, symbol: SymbolRef) -> TypeId {
        let error = self.types.intrinsics().error;
        if symbol == self.special.unknown {
            return error;
        }
        if self
            .symbol_flags(symbol)
            .intersects(SymbolFlags::TYPE_ALIAS)
        {
            return self.type_from_type_alias_reference(node, symbol);
        }
        error
    }

    fn type_from_type_alias_reference(&mut self, node: NodeRef, symbol: SymbolRef) -> TypeId {
        let ty = self.declared_type_of_type_alias(symbol);
        if self.check_no_type_arguments(node, symbol) {
            ty
        } else {
            self.types.intrinsics().error
        }
    }

    /// Reports type arguments on a reference to a non-generic type.
    fn check_no_type_arguments(&mut self, node: NodeRef, symbol: SymbolRef) -> bool {
        let ast = self.files[node.file].parsed().ast();
        let has_arguments = ast
            .node(node.node)
            .data()
            .type_arguments()
            .is_some_and(|arguments| !ast.list(arguments).is_empty());
        if has_arguments {
            let name = self.symbol_name(symbol).to_owned();
            self.error_on(node, diagnostics::TYPE_0_IS_NOT_GENERIC, &[&name]);
            return false;
        }
        true
    }

    /// Returns the type a type alias stands for, reporting aliases that reference themselves.
    pub(super) fn declared_type_of_type_alias(&mut self, symbol: SymbolRef) -> TypeId {
        if let Some(&ty) = self.type_alias_declared_types.get(&symbol) {
            return ty;
        }
        let error = self.types.intrinsics().error;
        if !self.push_type_resolution(symbol, TypeSystemProperty::DeclaredType) {
            return error;
        }
        let declaration = self
            .symbol_declarations(symbol)
            .into_iter()
            .find(|declaration| {
                matches!(
                    self.files[declaration.file]
                        .parsed()
                        .ast()
                        .node(declaration.node)
                        .kind(),
                    SyntaxKind::TypeAliasDeclaration | SyntaxKind::JSTypeAliasDeclaration
                )
            });
        let Some(declaration) = declaration else {
            self.pop_type_resolution();
            return error;
        };
        let ast = self.files[declaration.file].parsed().ast();
        let NodeData::TypeAliasDeclaration(alias) = ast.node(declaration.node).data() else {
            unreachable!("a type alias declaration carries alias data");
        };
        let (name, type_node) = (alias.name, alias.type_node);
        let ty = self.type_from_type_node(NodeRef {
            file: declaration.file,
            node: type_node,
        });
        let ty = if self.pop_type_resolution() {
            ty
        } else {
            let name_text = self.symbol_name(symbol).to_owned();
            self.error_on(
                NodeRef {
                    file: declaration.file,
                    node: name,
                },
                diagnostics::TYPE_ALIAS_0_CIRCULARLY_REFERENCES_ITSELF,
                &[&name_text],
            );
            error
        };
        *self.type_alias_declared_types.entry(symbol).or_insert(ty)
    }

    /// Starts resolving `property` of `target`; returns false, marking every resolution since the
    /// earlier start as circular, when that resolution is already in progress.
    fn push_type_resolution(&mut self, target: SymbolRef, property: TypeSystemProperty) -> bool {
        if let Some(start) = self.resolution_cycle_start(target, property) {
            for resolution in &mut self.type_resolutions[start..] {
                resolution.result = false;
            }
            return false;
        }
        self.type_resolutions.push(TypeResolution {
            target,
            property,
            result: true,
        });
        true
    }

    /// Finishes the innermost resolution; returns whether it completed without a cycle.
    fn pop_type_resolution(&mut self) -> bool {
        self.type_resolutions
            .pop()
            .expect("pops match pushes")
            .result
    }

    fn resolution_cycle_start(
        &self,
        target: SymbolRef,
        property: TypeSystemProperty,
    ) -> Option<usize> {
        for (index, resolution) in self.type_resolutions.iter().enumerate().rev() {
            if self.type_resolution_has_property(resolution) {
                return None;
            }
            if resolution.target == target && resolution.property == property {
                return Some(index);
            }
        }
        None
    }

    /// Returns whether a resolution's property has meanwhile been computed.
    fn type_resolution_has_property(&self, resolution: &TypeResolution) -> bool {
        match resolution.property {
            TypeSystemProperty::DeclaredType => self
                .type_alias_declared_types
                .contains_key(&resolution.target),
        }
    }
}
