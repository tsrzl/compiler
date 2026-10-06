//! Lexical name resolution, modeled on TypeScript-Go's `binder.NameResolver.Resolve`.
//!
//! Resolution walks from a location up through its containers, consulting each scope's locals,
//! module and enum exports, class type parameters, and finally the globals.
//!
//! Not yet ported: alias target flags (aliases match only their own flags until module
//! resolution exists), unused-symbol tracking, the property-initializer and failure callbacks,
//! and target-dependent parameter scopes, which assume an `ESNext` target.

use super::checker::Checker;
use super::program::{CheckDiagnostic, NodeRef};
use super::symbol_store::SymbolRef;
use crate::ast::{Ast, ModifierFlags, NodeData, NodeFlags, NodeId, SymbolFlags, SyntaxKind};
use crate::diagnostics::{self, Message};
use crate::symbols::{INTERNAL_SYMBOL_NAME_DEFAULT, SymbolTable};

/// The outcome of a resolution walk with the errors it found along the way.
struct Resolution {
    symbol: Option<SymbolRef>,
    errors: Vec<CheckDiagnostic>,
    /// Whether the walk stopped early, so a missing symbol is not reported as unresolved.
    stopped: bool,
}

impl Checker<'_> {
    /// Resolves `name` with `meaning` as seen from `location`, without reporting errors.
    #[must_use]
    pub fn resolve_name(
        &self,
        location: NodeRef,
        name: &str,
        meaning: SymbolFlags,
        _: Option<Message>,
    ) -> Option<SymbolRef> {
        self.resolve_name_worker(location, name, meaning, false, LookupMode::Exact)
            .symbol
    }

    /// Resolves `name` and reports scope errors, such as a static member referencing a class
    /// type parameter. `name_not_found_message` is the message for an unresolved name.
    pub fn resolve_name_reporting(
        &mut self,
        location: NodeRef,
        name: &str,
        meaning: SymbolFlags,
        name_not_found_message: Message,
    ) -> Option<SymbolRef> {
        let resolution = self.resolve_name_worker(location, name, meaning, true, LookupMode::Exact);
        let failed = resolution.symbol.is_none() && !resolution.stopped;
        for error in resolution.errors {
            self.add_diagnostic(error);
        }
        if failed {
            self.on_failed_to_resolve_symbol(location, name, meaning, name_not_found_message);
        }
        resolution.symbol
    }

    /// Resolves `name` with the given lookup mode, without reporting errors.
    pub(super) fn resolve_name_with_mode(
        &self,
        location: NodeRef,
        name: &str,
        meaning: SymbolFlags,
        mode: LookupMode,
    ) -> Option<SymbolRef> {
        self.resolve_name_worker(location, name, meaning, false, mode)
            .symbol
    }

    fn resolve_name_worker(
        &self,
        original: NodeRef,
        name: &str,
        meaning: SymbolFlags,
        report: bool,
        mode: LookupMode,
    ) -> Resolution {
        let file = original.file;
        let ast = self.files[file].parsed().ast();
        let mut errors = Vec::new();
        // Scope errors are reported at the reference being resolved.
        let mut error = |message: Message| {
            if report {
                errors.push(self.diagnostic_for(original, message, &[]));
            }
        };
        let mut location = Some(original.node);
        let mut last_location: Option<NodeId> = None;
        while let Some(mut current) = location {
            if name == "const" && is_const_assertion(ast, current) {
                // `const` in `as const` names no symbol and needs no lookup.
                return Resolution {
                    symbol: None,
                    errors,
                    stopped: true,
                };
            }
            if matches!(
                ast.node(current).kind(),
                SyntaxKind::ModuleDeclaration | SyntaxKind::EnumDeclaration
            ) && last_location.is_some()
                && ast.node(current).data().name() == last_location
            {
                // The name of a namespace or enum resolves outside its own scope.
                last_location = Some(current);
                match ast.node(current).parent() {
                    Some(parent) => current = parent,
                    None => break,
                }
            }
            if let Some(symbol) =
                self.lookup_in_locals(file, current, name, meaning, last_location, mode)
            {
                return Resolution {
                    symbol: Some(symbol),
                    errors,
                    stopped: false,
                };
            }
            match self.lookup_in_container(
                &Scope {
                    file,
                    location: current,
                    name,
                    meaning,
                    last_location,
                    mode,
                },
                &mut error,
            ) {
                ContainerLookup::Found(symbol) => {
                    return Resolution {
                        symbol: Some(symbol),
                        errors,
                        stopped: false,
                    };
                }
                ContainerLookup::Stop => {
                    return Resolution {
                        symbol: None,
                        errors,
                        stopped: true,
                    };
                }
                ContainerLookup::Continue(next) => current = next,
            }
            last_location = Some(current);
            location = ast.node(current).parent();
        }
        let symbol = self.lookup_mode(
            mode,
            &self.globals,
            name,
            meaning | SymbolFlags::GLOBAL_LOOKUP,
        );
        Resolution {
            symbol,
            errors,
            stopped: false,
        }
    }

    /// Looks `name` up in a scope's own locals, applying the visibility rules of function
    /// signatures and conditional types.
    fn lookup_in_locals(
        &self,
        file: usize,
        location: NodeId,
        name: &str,
        meaning: SymbolFlags,
        last_location: Option<NodeId>,
        mode: LookupMode,
    ) -> Option<SymbolRef> {
        let program_file = &self.files[file];
        let ast = program_file.parsed().ast();
        // A script's locals are merged into the globals and are not in scope here.
        if ast.node(location).kind() == SyntaxKind::SourceFile && !program_file.is_external_module()
        {
            return None;
        }
        let locals = program_file.bound().locals(location)?;
        let symbol = self.lookup_bound(mode, file, locals, name, meaning)?;
        let flags = self.symbol_flags(symbol);
        let kind = ast.node(location).kind();
        let use_result = if is_function_like_kind(kind)
            && let Some(last) = last_location
            && Some(last) != ast.node(location).data().body()
        {
            self.is_visible_in_signature(file, location, last, symbol, flags, meaning)
        } else if let Some(conditional) = ast.node(location).data().as_conditional_type_node() {
            // An `infer T` type parameter is visible only in the true branch.
            last_location == Some(conditional.true_type)
        } else {
            true
        };
        use_result.then_some(symbol)
    }

    /// Applies function signature scoping: type parameters are visible in the parameter list
    /// and return type, local types only in the body, and parameters in the parameter list,
    /// return type, and body.
    fn is_visible_in_signature(
        &self,
        file: usize,
        location: NodeId,
        last: NodeId,
        symbol: SymbolRef,
        flags: SymbolFlags,
        meaning: SymbolFlags,
    ) -> bool {
        let ast = self.files[file].parsed().ast();
        let last_kind = ast.node(last).kind();
        let synthesized = ast.node(last).flags().intersects(NodeFlags::SYNTHESIZED);
        let in_return_type = ast.node(location).data().type_node() == Some(last);
        let mut visible = true;
        if (meaning & flags).intersects(SymbolFlags::TYPE) && last_kind != SyntaxKind::JSDoc {
            visible = flags.intersects(SymbolFlags::TYPE_PARAMETER)
                && (synthesized
                    || in_return_type
                    || matches!(
                        last_kind,
                        SyntaxKind::Parameter
                            | SyntaxKind::JSDocParameterTag
                            | SyntaxKind::JSDocReturnTag
                            | SyntaxKind::TypeParameter
                    ));
        }
        if (meaning & flags).intersects(SymbolFlags::VARIABLE) {
            if self.use_outer_variable_scope_in_parameter(file, symbol, location, last) {
                visible = false;
            } else if flags.intersects(SymbolFlags::FUNCTION_SCOPED_VARIABLE) {
                let declared_in_parameter =
                    self.value_declaration(symbol).is_some_and(|declaration| {
                        has_parameter_ancestor(
                            self.files[declaration.file].parsed().ast(),
                            declaration.node,
                        )
                    });
                visible = last_kind == SyntaxKind::Parameter
                    || synthesized
                    || in_return_type && declared_in_parameter;
            }
        }
        visible
    }

    /// Returns whether a parameter initializer referring to a body declaration should see the
    /// outer scope instead. With an `ESNext` target no parameter requires a scope change, so this
    /// holds whenever the declaration is inside the body.
    fn use_outer_variable_scope_in_parameter(
        &self,
        file: usize,
        symbol: SymbolRef,
        location: NodeId,
        last: NodeId,
    ) -> bool {
        let ast = self.files[file].parsed().ast();
        if ast.node(last).kind() != SyntaxKind::Parameter {
            return false;
        }
        let (Some(body), Some(declaration)) = (
            ast.node(location).data().body(),
            self.value_declaration(symbol),
        ) else {
            return false;
        };
        let declaration = self.files[declaration.file]
            .parsed()
            .ast()
            .node(declaration.node);
        declaration.pos() >= ast.node(body).pos() && declaration.end() <= ast.node(body).end()
    }

    fn lookup_in_container(
        &self,
        scope: &Scope<'_>,
        error: &mut impl FnMut(Message),
    ) -> ContainerLookup {
        let program_file = &self.files[scope.file];
        let ast = program_file.parsed().ast();
        let (location, meaning) = (scope.location, scope.meaning);
        match ast.node(location).kind() {
            SyntaxKind::SourceFile if program_file.is_external_module() => {
                self.lookup_in_module(scope)
            }
            SyntaxKind::ModuleDeclaration => self.lookup_in_module(scope),
            SyntaxKind::EnumDeclaration => self
                .symbol_of_declaration(scope.file, location)
                .and_then(|symbol| {
                    let exports = self.symbols.view(self.files, symbol).exports;
                    self.lookup_mode(
                        scope.mode,
                        &exports,
                        scope.name,
                        meaning & SymbolFlags::ENUM_MEMBER,
                    )
                })
                .map_or(ContainerLookup::Continue(location), ContainerLookup::Found),
            SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::InterfaceDeclaration => self.lookup_in_class_like(scope, error),
            SyntaxKind::ExpressionWithTypeArguments => {
                self.check_base_class_expression(scope, error)
            }
            SyntaxKind::ComputedPropertyName => self.check_computed_property_name(scope, error),
            SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::FunctionDeclaration
                if meaning.intersects(SymbolFlags::VARIABLE) && scope.name == "arguments" =>
            {
                ContainerLookup::Found(self.special.arguments)
            }
            SyntaxKind::FunctionExpression => {
                if meaning.intersects(SymbolFlags::VARIABLE) && scope.name == "arguments" {
                    return ContainerLookup::Found(self.special.arguments);
                }
                if meaning.intersects(SymbolFlags::FUNCTION) {
                    return self.lookup_own_name(scope, location);
                }
                ContainerLookup::Continue(location)
            }
            SyntaxKind::Decorator => {
                ContainerLookup::Continue(decorator_resolution_location(ast, location))
            }
            SyntaxKind::InferType if meaning.intersects(SymbolFlags::TYPE_PARAMETER) => {
                let parameter = ast
                    .node(location)
                    .data()
                    .as_infer_type_node()
                    .map(|infer| infer.type_parameter)
                    .expect("an infer type declares a type parameter");
                match self.lookup_own_name(scope, parameter) {
                    ContainerLookup::Found(symbol) => ContainerLookup::Found(symbol),
                    _ => ContainerLookup::Continue(location),
                }
            }
            SyntaxKind::ExportSpecifier => {
                ContainerLookup::Continue(export_specifier_scope(ast, scope))
            }
            _ => ContainerLookup::Continue(location),
        }
    }

    /// Resolves a class or interface's own type parameters, which static members cannot see,
    /// and a class expression's own name.
    fn lookup_in_class_like(
        &self,
        scope: &Scope<'_>,
        error: &mut impl FnMut(Message),
    ) -> ContainerLookup {
        let ast = self.files[scope.file].parsed().ast();
        let location = scope.location;
        if let Some(symbol) = self.lookup_type_parameter_of(
            scope.file,
            location,
            scope.name,
            scope.meaning,
            scope.mode,
        ) {
            if scope.last_location.is_some_and(|last| is_static(ast, last)) {
                error(diagnostics::STATIC_MEMBERS_CANNOT_REFERENCE_CLASS_TYPE_PARAMETERS);
                return ContainerLookup::Stop;
            }
            return ContainerLookup::Found(symbol);
        }
        if ast.node(location).kind() == SyntaxKind::ClassExpression
            && scope.meaning.intersects(SymbolFlags::CLASS)
        {
            return self.lookup_own_name(scope, location);
        }
        ContainerLookup::Continue(location)
    }

    /// Reports a base class expression that references its class's type parameters.
    fn check_base_class_expression(
        &self,
        scope: &Scope<'_>,
        error: &mut impl FnMut(Message),
    ) -> ContainerLookup {
        let ast = self.files[scope.file].parsed().ast();
        let location = scope.location;
        let extends_class = ast.node(location).parent().and_then(|clause| {
            let heritage = ast.node(clause).data().as_heritage_clause()?;
            (heritage.token == SyntaxKind::ExtendsKeyword
                && scope.last_location == ast.node(location).data().expression())
            .then(|| ast.node(clause).parent())
            .flatten()
            .filter(|&container| is_class_like(ast, container))
        });
        if let Some(container) = extends_class
            && self
                .lookup_type_parameter_of(
                    scope.file,
                    container,
                    scope.name,
                    scope.meaning,
                    scope.mode,
                )
                .is_some()
        {
            error(diagnostics::BASE_CLASS_EXPRESSIONS_CANNOT_REFERENCE_CLASS_TYPE_PARAMETERS);
            return ContainerLookup::Stop;
        }
        ContainerLookup::Continue(location)
    }

    /// Reports a computed member name that references its containing type's type parameters.
    fn check_computed_property_name(
        &self,
        scope: &Scope<'_>,
        error: &mut impl FnMut(Message),
    ) -> ContainerLookup {
        let ast = self.files[scope.file].parsed().ast();
        let location = scope.location;
        let container = ast
            .node(location)
            .parent()
            .and_then(|member| ast.node(member).parent())
            .filter(|&container| {
                is_class_like(ast, container)
                    || ast.node(container).kind() == SyntaxKind::InterfaceDeclaration
            });
        if let Some(container) = container
            && self
                .lookup_type_parameter_of(
                    scope.file,
                    container,
                    scope.name,
                    scope.meaning,
                    scope.mode,
                )
                .is_some()
        {
            error(diagnostics::A_COMPUTED_PROPERTY_NAME_CANNOT_REFERENCE_A_TYPE_PARAMETER_FROM_ITS_CONTAINING_TYPE);
            return ContainerLookup::Stop;
        }
        ContainerLookup::Continue(location)
    }

    /// Resolves to `declaration`'s own symbol when its name is the name being resolved.
    fn lookup_own_name(&self, scope: &Scope<'_>, declaration: NodeId) -> ContainerLookup {
        let ast = self.files[scope.file].parsed().ast();
        let own_name = ast
            .node(declaration)
            .data()
            .name()
            .and_then(|name| ast.identifier_text(name));
        if own_name == Some(scope.name)
            && let Some(symbol) = self.symbol_of_declaration(scope.file, declaration)
        {
            return ContainerLookup::Found(symbol);
        }
        ContainerLookup::Continue(scope.location)
    }

    /// Looks `name` up among a module's exports: a matching local `export default` name first,
    /// then other exports, skipping names that are only export specifiers.
    fn lookup_in_module(&self, scope: &Scope<'_>) -> ContainerLookup {
        let (file, location, name, meaning) =
            (scope.file, scope.location, scope.name, scope.meaning);
        let ast = self.files[file].parsed().ast();
        let Some(module) = self.symbol_of_declaration(file, location) else {
            return ContainerLookup::Continue(location);
        };
        let exports = self.symbols.view(self.files, module).exports;
        let is_external = ast.node(location).kind() == SyntaxKind::SourceFile
            || ast.node(location).flags().intersects(NodeFlags::AMBIENT)
                && !ast.is_global_scope_augmentation(location);
        if is_external {
            if let Some(default) = exports.get(INTERNAL_SYMBOL_NAME_DEFAULT)
                && self.symbol_flags(default).intersects(meaning)
                && self
                    .local_symbol_for_export_default(default)
                    .is_some_and(|local| self.symbol_name(local) == name)
            {
                return ContainerLookup::Found(default);
            }
            if let Some(export) = exports.get(name)
                && self.symbol_flags(export) == SymbolFlags::ALIAS
                && self.symbol_declarations(export).iter().any(|declaration| {
                    matches!(
                        self.files[declaration.file]
                            .parsed()
                            .ast()
                            .node(declaration.node)
                            .kind(),
                        SyntaxKind::ExportSpecifier | SyntaxKind::NamespaceExport
                    )
                })
            {
                return ContainerLookup::Continue(location);
            }
        }
        if name != INTERNAL_SYMBOL_NAME_DEFAULT
            && let Some(symbol) = self.lookup_mode(
                scope.mode,
                &exports,
                name,
                meaning & SymbolFlags::MODULE_MEMBER,
            )
        {
            return ContainerLookup::Found(symbol);
        }
        ContainerLookup::Continue(location)
    }

    /// Returns a type parameter named `name` declared directly by the class or interface.
    fn lookup_type_parameter_of(
        &self,
        file: usize,
        container: NodeId,
        name: &str,
        meaning: SymbolFlags,
        mode: LookupMode,
    ) -> Option<SymbolRef> {
        let symbol = self.symbol_of_declaration(file, container)?;
        let members = self.symbols.view(self.files, symbol).members;
        let result = self.lookup_mode(mode, &members, name, meaning & SymbolFlags::TYPE)?;
        self.symbol_declarations(result)
            .iter()
            .any(|declaration| {
                let ast = self.files[declaration.file].parsed().ast();
                declaration.file == file
                    && ast.node(declaration.node).kind() == SyntaxKind::TypeParameter
                    && ast.node(declaration.node).parent() == Some(container)
            })
            .then_some(result)
    }

    /// Returns the local symbol paired with an `export default` declaration's export symbol.
    fn local_symbol_for_export_default(&self, symbol: SymbolRef) -> Option<SymbolRef> {
        let declarations = self.symbol_declarations(symbol);
        let first = declarations.first()?;
        let first_ast = self.files[first.file].parsed().ast();
        if !first_ast.has_syntactic_modifier(first.node, ModifierFlags::DEFAULT) {
            return None;
        }
        declarations.iter().find_map(|declaration| {
            self.files[declaration.file]
                .bound()
                .local_symbol_of(declaration.node)
                .map(|local| SymbolRef::Bound {
                    file: declaration.file,
                    symbol: local,
                })
        })
    }

    /// Returns the merged symbol a declaration contributes to.
    pub(super) fn symbol_of_declaration(&self, file: usize, node: NodeId) -> Option<SymbolRef> {
        let symbol = self.files[file].bound().symbol_of(node)?;
        Some(self.merged_symbol(SymbolRef::Bound { file, symbol }))
    }

    pub(super) fn value_declaration(&self, symbol: SymbolRef) -> Option<NodeRef> {
        self.symbols.view(self.files, symbol).value_declaration
    }

    /// Returns the merged symbol named `name` whose flags include `meaning`.
    pub(super) fn lookup(
        &self,
        table: &SymbolTable<SymbolRef>,
        name: &str,
        meaning: SymbolFlags,
    ) -> Option<SymbolRef> {
        if meaning.bits() == 0 {
            return None;
        }
        let symbol = self.merged_symbol(table.get(name)?);
        self.symbol_flags(symbol)
            .intersects(meaning)
            .then_some(symbol)
    }

    /// Looks a bound locals table up by converting it to program-wide references.
    fn lookup_bound(
        &self,
        mode: LookupMode,
        file: usize,
        table: &SymbolTable,
        name: &str,
        meaning: SymbolFlags,
    ) -> Option<SymbolRef> {
        let mut converted = SymbolTable::default();
        if mode == LookupMode::Exact {
            converted.insert(
                name,
                SymbolRef::Bound {
                    file,
                    symbol: table.get(name)?,
                },
            );
        } else {
            for (entry, symbol) in table.iter() {
                converted.insert(entry, SymbolRef::Bound { file, symbol });
            }
        }
        self.lookup_mode(mode, &converted, name, meaning)
    }

    /// Looks `name` up exactly or, while finding a suggestion, falls back to the closest
    /// spelling in the table, as TypeScript-Go's `getSuggestionForSymbolNameLookup` does.
    fn lookup_mode(
        &self,
        mode: LookupMode,
        table: &SymbolTable<SymbolRef>,
        name: &str,
        meaning: SymbolFlags,
    ) -> Option<SymbolRef> {
        let exact = self.lookup(table, name, meaning);
        if mode == LookupMode::Exact || exact.is_some() {
            return exact;
        }
        let extras = if meaning.intersects(SymbolFlags::GLOBAL_LOOKUP) {
            self.primitive_type_alias_suggestions(table)
        } else {
            Vec::new()
        };
        let candidates = table
            .iter()
            .map(|(_, symbol)| self.merged_symbol(symbol))
            .chain(extras);
        self.spelling_suggestion_for_name(name, candidates, meaning)
    }
}

/// Whether a lookup must match exactly or may suggest a close spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LookupMode {
    Exact,
    Suggestion,
}

/// One step of a resolution walk.
struct Scope<'name> {
    mode: LookupMode,
    file: usize,
    location: NodeId,
    name: &'name str,
    meaning: SymbolFlags,
    last_location: Option<NodeId>,
}

/// What a container contributed to a resolution walk.
enum ContainerLookup {
    /// The name resolved.
    Found(SymbolRef),
    /// Resolution failed with an error and stops.
    Stop,
    /// Resolution continues from the given node's parent.
    Continue(NodeId),
}

/// Returns where resolution continues from an export specifier: names re-exported from another
/// module, as `a` in `export { a as b } from "m"`, resolve outside the export declaration.
fn export_specifier_scope(ast: &Ast, scope: &Scope<'_>) -> NodeId {
    let location = scope.location;
    let specifier = ast
        .node(location)
        .data()
        .as_export_specifier()
        .expect("checked by kind");
    let declaration = ast
        .node(location)
        .parent()
        .and_then(|list| ast.node(list).parent());
    let reexports_module = declaration
        .and_then(|declaration| ast.node(declaration).data().as_export_declaration())
        .is_some_and(|declaration| declaration.module_specifier.is_some());
    if scope.last_location.is_some()
        && scope.last_location == specifier.property_name
        && reexports_module
    {
        return declaration
            .and_then(|declaration| ast.node(declaration).parent())
            .unwrap_or(location);
    }
    location
}

fn is_const_assertion(ast: &Ast, node: NodeId) -> bool {
    let type_node = match ast.node(node).data() {
        NodeData::AsExpression(expression) => expression.type_node,
        NodeData::TypeAssertion(assertion) => assertion.type_node,
        _ => return false,
    };
    ast.node(type_node)
        .data()
        .as_type_reference_node()
        .is_some_and(|reference| {
            reference
                .type_arguments
                .is_none_or(|arguments| ast.list(arguments).is_empty())
                && ast.identifier_text(reference.type_name) == Some("const")
        })
}

const fn is_function_like_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::MethodSignature
            | SyntaxKind::CallSignature
            | SyntaxKind::JSDocSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::IndexSignature
            | SyntaxKind::FunctionType
            | SyntaxKind::ConstructorType
    )
}

fn has_parameter_ancestor(ast: &Ast, node: NodeId) -> bool {
    std::iter::successors(Some(node), |&id| ast.node(id).parent())
        .any(|id| ast.node(id).kind() == SyntaxKind::Parameter)
}

fn is_class_like(ast: &Ast, node: NodeId) -> bool {
    matches!(
        ast.node(node).kind(),
        SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
    )
}

fn is_static(ast: &Ast, node: NodeId) -> bool {
    let kind = ast.node(node).kind();
    let is_class_element = matches!(
        kind,
        SyntaxKind::Constructor
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::IndexSignature
            | SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::SemicolonClassElement
    );
    is_class_element && ast.has_syntactic_modifier(node, ModifierFlags::STATIC)
        || kind == SyntaxKind::ClassStaticBlockDeclaration
}

/// Returns where a decorator's names resolve: parameter and member decorators at the class,
/// and class decorators outside the class.
fn decorator_resolution_location(ast: &Ast, decorator: NodeId) -> NodeId {
    let mut location = decorator;
    if let Some(parent) = ast.node(location).parent()
        && ast.node(parent).kind() == SyntaxKind::Parameter
    {
        location = parent;
    }
    if let Some(parent) = ast.node(location).parent()
        && (is_class_element_kind(ast.node(parent).kind())
            || ast.node(parent).kind() == SyntaxKind::ClassDeclaration)
    {
        location = parent;
    }
    location
}

const fn is_class_element_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Constructor
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::IndexSignature
            | SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::SemicolonClassElement
    )
}
