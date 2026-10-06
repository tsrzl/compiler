//! Namespace instantiation analysis modeled on TypeScript-Go's `GetModuleInstanceState`.

use std::collections::HashMap;

use super::{Ast, ModifierFlags, NodeData, NodeId, SyntaxKind};

/// Whether a namespace declaration produces a runtime value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModuleInstanceState {
    /// Not yet determined; used to break cycles.
    Unknown,
    /// The namespace contains only types, so it has no runtime value.
    NonInstantiated,
    /// The namespace contains values.
    Instantiated,
    /// The namespace contains only const enums, which are inlined.
    ConstEnumOnly,
}

impl Ast {
    /// Returns whether the namespace `node` produces a runtime value.
    ///
    /// # Panics
    ///
    /// Panics when `node` is not a module declaration.
    #[must_use]
    pub fn module_instance_state(&self, node: NodeId) -> ModuleInstanceState {
        module_instance_state(self, node, &mut HashMap::new())
    }
}

type Visited = HashMap<NodeId, ModuleInstanceState>;

fn module_instance_state(ast: &Ast, node: NodeId, visited: &mut Visited) -> ModuleInstanceState {
    let module = ast
        .node(node)
        .data()
        .as_module_declaration()
        .expect("module instance state is computed for module declarations");
    module
        .body
        .map_or(ModuleInstanceState::Instantiated, |body| {
            cached_state(ast, body, visited)
        })
}

fn cached_state(ast: &Ast, node: NodeId, visited: &mut Visited) -> ModuleInstanceState {
    if let Some(&cached) = visited.get(&node) {
        return if cached == ModuleInstanceState::Unknown {
            ModuleInstanceState::NonInstantiated
        } else {
            cached
        };
    }
    visited.insert(node, ModuleInstanceState::Unknown);
    let state = state_worker(ast, node, visited);
    visited.insert(node, state);
    state
}

fn state_worker(ast: &Ast, node: NodeId, visited: &mut Visited) -> ModuleInstanceState {
    match ast.node(node).data() {
        _ if matches!(
            ast.node(node).kind(),
            SyntaxKind::InterfaceDeclaration
                | SyntaxKind::TypeAliasDeclaration
                | SyntaxKind::JSTypeAliasDeclaration
        ) =>
        {
            ModuleInstanceState::NonInstantiated
        }
        NodeData::EnumDeclaration(_)
            if ast
                .combined_modifier_flags(node)
                .intersects(ModifierFlags::CONST) =>
        {
            ModuleInstanceState::ConstEnumOnly
        }
        NodeData::ImportDeclaration(_) | NodeData::ImportEqualsDeclaration(_)
            if !ast.has_syntactic_modifier(node, ModifierFlags::EXPORT) =>
        {
            ModuleInstanceState::NonInstantiated
        }
        NodeData::ExportDeclaration(declaration)
            if declaration.module_specifier.is_none()
                && declaration
                    .export_clause
                    .is_some_and(|clause| ast.node(clause).kind() == SyntaxKind::NamedExports) =>
        {
            let clause = declaration.export_clause.expect("checked by the guard");
            let elements = ast
                .node(clause)
                .data()
                .as_named_exports()
                .expect("a named exports clause carries named exports data")
                .elements;
            let mut state = ModuleInstanceState::NonInstantiated;
            for &specifier in ast.list(elements) {
                state = state.max(alias_target_state(ast, specifier, visited));
                if state == ModuleInstanceState::Instantiated {
                    break;
                }
            }
            state
        }
        NodeData::ModuleBlock(_) => {
            let mut state = ModuleInstanceState::NonInstantiated;
            for child in ast.children(node) {
                match cached_state(ast, child, visited) {
                    ModuleInstanceState::NonInstantiated => {}
                    ModuleInstanceState::ConstEnumOnly => {
                        state = ModuleInstanceState::ConstEnumOnly;
                    }
                    ModuleInstanceState::Instantiated => return ModuleInstanceState::Instantiated,
                    ModuleInstanceState::Unknown => {
                        unreachable!("cached module instance states are never unknown")
                    }
                }
            }
            state
        }
        NodeData::ModuleDeclaration(_) => module_instance_state(ast, node, visited),
        _ => ModuleInstanceState::Instantiated,
    }
}

/// Returns the state of the local declaration an export specifier re-exports.
fn alias_target_state(ast: &Ast, specifier: NodeId, visited: &mut Visited) -> ModuleInstanceState {
    let export = ast
        .node(specifier)
        .data()
        .as_export_specifier()
        .expect("named exports contain export specifiers");
    let name = export.property_name.unwrap_or(export.name);
    let Some(name) = ast.identifier_text(name) else {
        // Invalid syntax such as `export { "x" }`.
        return ModuleInstanceState::Instantiated;
    };
    let mut ancestor = ast.node(specifier).parent();
    while let Some(scope) = ancestor {
        if let Some(statements) = scope_statements(ast, scope) {
            let mut found = ModuleInstanceState::Unknown;
            for &statement in ast.list(statements) {
                if !node_has_name(ast, statement, name) {
                    continue;
                }
                let state = cached_state(ast, statement, visited);
                if found == ModuleInstanceState::Unknown || state > found {
                    found = state;
                }
                if found == ModuleInstanceState::Instantiated {
                    return found;
                }
                // Re-exports of import aliases are ambiguous, so they count as instantiated,
                // consistent with `export import x = mod.x`.
                if ast.node(statement).kind() == SyntaxKind::ImportEqualsDeclaration {
                    found = ModuleInstanceState::Instantiated;
                }
            }
            if found != ModuleInstanceState::Unknown {
                return found;
            }
        }
        ancestor = ast.node(scope).parent();
    }
    // The name could not be located, so it may refer to a value.
    ModuleInstanceState::Instantiated
}

fn scope_statements(ast: &Ast, node: NodeId) -> Option<super::NodeList> {
    match ast.node(node).data() {
        NodeData::Block(block) => Some(block.statements),
        NodeData::ModuleBlock(block) => Some(block.statements),
        NodeData::SourceFile(file) => Some(file.statements),
        _ => None,
    }
}

/// Returns whether a statement declares an identifier named `name`.
fn node_has_name(ast: &Ast, statement: NodeId, name: &str) -> bool {
    let data = ast.node(statement).data();
    if let Some(declared) = data.name() {
        return ast.identifier_text(declared) == Some(name);
    }
    if let NodeData::VariableStatement(variables) = data {
        let list = ast
            .node(variables.declaration_list)
            .data()
            .as_variable_declaration_list()
            .expect("a variable statement owns a declaration list");
        return ast
            .list(list.declarations)
            .iter()
            .any(|&declaration| node_has_name(ast, declaration, name));
    }
    false
}
