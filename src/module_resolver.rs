//! Resolves module specifiers against source files in the current compilation.

use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

use crate::syntax::{Diagnostic, ImportDeclaration, Statement, SyntaxTree, TextSpan};

pub(crate) fn check(syntax_trees: &[SyntaxTree]) -> Vec<Diagnostic> {
    syntax_trees
        .iter()
        .flat_map(|syntax_tree| {
            syntax_tree
                .program()
                .statements()
                .iter()
                .flat_map(|statement| {
                    check_statement(syntax_tree.source_file().path(), statement, syntax_trees)
                })
                .map(|diagnostic| diagnostic.in_file(syntax_tree.file_id()))
        })
        .collect()
}

fn check_statement(
    importer_path: &Path,
    statement: &Statement,
    syntax_trees: &[SyntaxTree],
) -> Vec<Diagnostic> {
    match statement.declaration() {
        Statement::ExportNamedFrom(export) => {
            check_named_reexport(importer_path, export, syntax_trees)
        }
        Statement::ExportAll(export) => {
            resolve_in_compilation(importer_path, export.module_specifier(), syntax_trees)
                .is_none()
                .then(|| missing_module_diagnostic(export.module_specifier(), export.span()))
                .into_iter()
                .collect()
        }
        Statement::ImportDeclaration(import) => check_import(importer_path, import, syntax_trees),
        _ => Vec::new(),
    }
}

fn check_named_reexport(
    importer_path: &Path,
    export: &crate::syntax::ExportNamedFromDeclaration,
    syntax_trees: &[SyntaxTree],
) -> Vec<Diagnostic> {
    let Some(exported_tree) =
        resolve_in_compilation(importer_path, export.module_specifier(), syntax_trees)
    else {
        return vec![missing_module_diagnostic(
            export.module_specifier(),
            export.span(),
        )];
    };
    let exports = exported_names(exported_tree, syntax_trees);
    export
        .specifiers()
        .iter()
        .filter(|specifier| !exports.contains(specifier.local_name()))
        .map(|specifier| {
            Diagnostic::new(
                2305,
                format!(
                    "Module '\"{}\"' has no exported member '{}'.",
                    export.module_specifier(),
                    specifier.local_name()
                ),
                specifier.span(),
            )
        })
        .collect()
}

fn check_import(
    importer_path: &Path,
    import: &ImportDeclaration,
    syntax_trees: &[SyntaxTree],
) -> Vec<Diagnostic> {
    let Some(imported_tree) =
        resolve_in_compilation(importer_path, import.module_specifier(), syntax_trees)
    else {
        let (code, message) = if import.imported_bindings().next().is_none() {
            (
                2882,
                format!(
                    "Cannot find module or type declarations for side-effect import of '{}'.",
                    import.module_specifier()
                ),
            )
        } else {
            (
                2307,
                format!(
                    "Cannot find module '{}' or its corresponding type declarations.",
                    import.module_specifier()
                ),
            )
        };
        return vec![Diagnostic::new(code, message, import.span())];
    };

    let exports = exported_names(imported_tree, syntax_trees);
    let mut diagnostics = Vec::new();
    if let Some(default_import) = import
        .default_import()
        .filter(|_| !exports.contains("default"))
    {
        diagnostics.push(Diagnostic::new(
            1192,
            format!(
                "Module '\"{}\"' has no default export.",
                import.module_specifier()
            ),
            default_import.span(),
        ));
    }
    diagnostics.extend(
        import
            .named_imports()
            .iter()
            .filter(|specifier| !exports.contains(specifier.imported_name()))
            .map(|specifier| {
                Diagnostic::new(
                    2305,
                    format!(
                        "Module '\"{}\"' has no exported member '{}'.",
                        import.module_specifier(),
                        specifier.imported_name()
                    ),
                    specifier.span(),
                )
            }),
    );
    diagnostics
}

fn missing_module_diagnostic(module_specifier: &str, span: TextSpan) -> Diagnostic {
    Diagnostic::new(
        2307,
        format!("Cannot find module '{module_specifier}' or its corresponding type declarations."),
        span,
    )
}

fn exported_names(syntax_tree: &SyntaxTree, syntax_trees: &[SyntaxTree]) -> HashSet<String> {
    let mut names = HashSet::new();
    collect_exported_names(syntax_tree, syntax_trees, &mut HashSet::new(), &mut names);
    names
}

fn collect_exported_names(
    syntax_tree: &SyntaxTree,
    syntax_trees: &[SyntaxTree],
    visited: &mut HashSet<PathBuf>,
    names: &mut HashSet<String>,
) {
    if !visited.insert(normalize_path(syntax_tree.source_file().path())) {
        return;
    }

    for statement in syntax_tree.program().statements() {
        if statement.is_exported() {
            if let Some(declaration) = statement.as_variable_declaration() {
                names.insert(declaration.name().to_owned());
            }
            if let Some(declaration) = statement.as_function_declaration() {
                names.insert(declaration.name().to_owned());
            }
            if let Some(declaration) = statement.as_class_declaration() {
                names.insert(declaration.name().to_owned());
            }
            if let Some(declaration) = statement.as_enum_declaration() {
                names.insert(declaration.name().to_owned());
            }
            if let Some(declaration) = statement.as_interface_declaration() {
                names.insert(declaration.name().to_owned());
            }
            if let Some(declaration) = statement.as_type_alias_declaration() {
                names.insert(declaration.name().to_owned());
            }
        }
        match statement.declaration() {
            Statement::ExportDefault(_) => {
                names.insert("default".to_owned());
            }
            Statement::ExportNamed(specifiers) => {
                names.extend(
                    specifiers
                        .iter()
                        .map(|specifier| specifier.exported_name().to_owned()),
                );
            }
            Statement::ExportTypeNamed(specifiers) => {
                names.extend(
                    specifiers
                        .iter()
                        .map(|specifier| specifier.exported_name().to_owned()),
                );
            }
            Statement::ExportNamedFrom(export) => {
                if let Some(exported_tree) = resolve_in_compilation(
                    syntax_tree.source_file().path(),
                    export.module_specifier(),
                    syntax_trees,
                ) {
                    let mut reexported_names = HashSet::new();
                    collect_exported_names(
                        exported_tree,
                        syntax_trees,
                        &mut visited.clone(),
                        &mut reexported_names,
                    );
                    names.extend(
                        export
                            .specifiers()
                            .iter()
                            .filter(|specifier| reexported_names.contains(specifier.local_name()))
                            .map(|specifier| specifier.exported_name().to_owned()),
                    );
                }
            }
            Statement::ExportAll(export) => {
                if let Some(exported_tree) = resolve_in_compilation(
                    syntax_tree.source_file().path(),
                    export.module_specifier(),
                    syntax_trees,
                ) {
                    let mut reexported_names = HashSet::new();
                    collect_exported_names(
                        exported_tree,
                        syntax_trees,
                        visited,
                        &mut reexported_names,
                    );
                    names.extend(
                        reexported_names
                            .into_iter()
                            .filter(|name| name != "default"),
                    );
                }
            }
            _ => {}
        }
    }
}

/// Resolves a module specifier to an existing source file beside its importer.
#[must_use]
pub fn resolve_from_disk(importer_path: &Path, module_specifier: &str) -> Option<PathBuf> {
    resolve_import(importer_path, module_specifier)
        .into_iter()
        .find(|candidate| candidate.is_file())
}

fn resolve_import(importer_path: &Path, module_specifier: &str) -> Vec<PathBuf> {
    let module_path = Path::new(module_specifier);
    let base_path = if module_path.is_absolute() {
        module_path.to_path_buf()
    } else if module_specifier.starts_with('.') {
        importer_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(module_path)
    } else {
        importer_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join("node_modules")
            .join(module_path)
    };

    candidate_paths(&base_path)
}

pub(crate) fn resolve_in_compilation<'trees>(
    importer_path: &Path,
    module_specifier: &str,
    syntax_trees: &'trees [SyntaxTree],
) -> Option<&'trees SyntaxTree> {
    resolve_import(importer_path, module_specifier)
        .iter()
        .map(|candidate| normalize_path(candidate))
        .find_map(|candidate| {
            syntax_trees
                .iter()
                .find(|syntax_tree| normalize_path(syntax_tree.source_file().path()) == candidate)
        })
}

fn candidate_paths(base_path: &Path) -> Vec<PathBuf> {
    let mut candidates = vec![base_path.to_path_buf()];
    let extensions = match base_path
        .extension()
        .and_then(|extension| extension.to_str())
    {
        None => &["ts", "tsx", "d.ts", "mts", "cts", "js", "jsx", "mjs", "cjs"][..],
        Some("js") => &["ts", "tsx", "d.ts", "js"][..],
        Some("jsx") => &["tsx", "jsx"][..],
        Some("mjs") => &["mts", "mjs"][..],
        Some("cjs") => &["cts", "cjs"][..],
        Some(_) => &[][..],
    };
    candidates.extend(
        extensions
            .iter()
            .map(|extension| base_path.with_extension(extension)),
    );

    if base_path.extension().is_none() {
        candidates.extend(
            ["ts", "tsx", "d.ts", "mts", "cts", "js", "jsx", "mjs", "cjs"]
                .iter()
                .map(|extension| base_path.join(format!("index.{extension}"))),
        );
    }

    candidates
}

pub(crate) fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir
                if matches!(
                    normalized.components().next_back(),
                    Some(Component::Normal(_))
                ) =>
            {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}
