//! External module detection modeled on TypeScript-Go's `internal/ast/parseoptions.go`.

use super::{ParsedSourceFile, ScriptKind};
use crate::ast::{Ast, ModifierFlags, NodeData, NodeFlags, NodeId, SyntaxKind};

/// Compiler-derived facts that can make a file a module without import or export syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExternalModuleIndicatorOptions {
    /// Whether JSX tags make a file a module, as with `react-jsx` and `react-jsxdev` emit.
    pub jsx: bool,
    /// Whether every non-declaration file is a module.
    pub force: bool,
}

impl ParsedSourceFile {
    /// Returns the node that makes this file an external module, if any.
    ///
    /// Import and export syntax or `import.meta` always indicate a module. Otherwise declaration
    /// files are scripts, and `options` decides whether JSX tags or the compiler settings force
    /// module semantics, in which case the source file node itself is the indicator.
    #[must_use]
    pub fn external_module_indicator(
        &self,
        options: ExternalModuleIndicatorOptions,
    ) -> Option<NodeId> {
        if self.options.script_kind() == ScriptKind::Json {
            return None;
        }
        let ast = &self.ast;
        if let Some(node) = probable_external_module_indicator(ast) {
            return Some(node);
        }
        if self.is_declaration_file {
            return None;
        }
        if options.jsx
            && let Some(node) = find_descendant(ast, ast.root(), is_jsx_tag)
        {
            return Some(node);
        }
        options.force.then(|| ast.root())
    }
}

fn probable_external_module_indicator(ast: &Ast) -> Option<NodeId> {
    let root = ast.node(ast.root());
    let statements = root.data().as_source_file().map(|file| file.statements)?;
    if let Some(&statement) = ast
        .list(statements)
        .iter()
        .find(|&&statement| is_external_module_indicator_node(ast, statement))
    {
        return Some(statement);
    }
    if root
        .flags()
        .intersects(NodeFlags::POSSIBLY_CONTAINS_IMPORT_META)
    {
        return find_descendant(ast, ast.root(), is_import_meta);
    }
    None
}

fn is_external_module_indicator_node(ast: &Ast, id: NodeId) -> bool {
    let data = ast.node(id).data();
    if data
        .modifiers()
        .is_some_and(|modifiers| modifiers.flags().intersects(ModifierFlags::EXPORT))
    {
        return true;
    }
    match data {
        NodeData::ImportEqualsDeclaration(declaration) => {
            ast.node(declaration.module_reference).kind() == SyntaxKind::ExternalModuleReference
        }
        NodeData::ImportDeclaration(_)
        | NodeData::ExportAssignment(_)
        | NodeData::ExportDeclaration(_) => true,
        _ => false,
    }
}

fn is_import_meta(ast: &Ast, id: NodeId) -> bool {
    ast.node(id).data().as_meta_property().is_some_and(|meta| {
        meta.keyword_token == SyntaxKind::ImportKeyword
            && ast
                .node(meta.name)
                .data()
                .as_identifier()
                .is_some_and(|name| &*name.text == "meta")
    })
}

fn is_jsx_tag(ast: &Ast, id: NodeId) -> bool {
    matches!(
        ast.node(id).kind(),
        SyntaxKind::JsxOpeningElement | SyntaxKind::JsxSelfClosingElement | SyntaxKind::JsxFragment
    )
}

/// Returns the first node in a pre-order walk below and including `id` that satisfies `check`.
fn find_descendant(ast: &Ast, id: NodeId, check: fn(&Ast, NodeId) -> bool) -> Option<NodeId> {
    if check(ast, id) {
        return Some(id);
    }
    let mut found = None;
    ast.for_each_child(id, |child| {
        found = find_descendant(ast, child, check);
        found.is_some()
    });
    found
}
