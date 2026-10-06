//! Symbol declaration and conflict reporting, modeled on TypeScript-Go's `declareSymbolEx`.

use super::binder::{Binder, Table};
use crate::ast::{ModifierFlags, NodeData, NodeId, SymbolFlags, SyntaxKind};
use crate::diagnostics::{self, Message};
use crate::scanner::token_to_string;
use crate::symbols::{
    INTERNAL_SYMBOL_NAME_CALL, INTERNAL_SYMBOL_NAME_COMPUTED, INTERNAL_SYMBOL_NAME_CONSTRUCTOR,
    INTERNAL_SYMBOL_NAME_DEFAULT, INTERNAL_SYMBOL_NAME_EXPORT_EQUALS,
    INTERNAL_SYMBOL_NAME_EXPORT_STAR, INTERNAL_SYMBOL_NAME_GLOBAL, INTERNAL_SYMBOL_NAME_INDEX,
    INTERNAL_SYMBOL_NAME_MISSING, INTERNAL_SYMBOL_NAME_NEW, INTERNAL_SYMBOL_NAME_PREFIX, SymbolId,
    SymbolTable,
};

impl Binder<'_> {
    /// Declares a symbol for `node` in `table`, merging with a compatible existing symbol and
    /// reporting a conflict on every declaration otherwise.
    ///
    /// `includes` are the flags the declaration adds; `excludes` are the flags it cannot be
    /// declared alongside in the same table.
    pub(super) fn declare_symbol(
        &mut self,
        table: Table,
        parent: Option<SymbolId>,
        node: NodeId,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> SymbolId {
        self.declare_symbol_ex(table, parent, node, includes, excludes, false)
    }

    pub(super) fn declare_symbol_ex(
        &mut self,
        table: Table,
        parent: Option<SymbolId>,
        node: NodeId,
        includes: SymbolFlags,
        excludes: SymbolFlags,
        is_computed_name: bool,
    ) -> SymbolId {
        debug_assert!(is_computed_name || !self.ast.has_dynamic_name(node));
        let is_default_export = self.is_default_export(node);
        // The exported symbol for an `export default` function or class is always named "default".
        let name = if is_computed_name {
            INTERNAL_SYMBOL_NAME_COMPUTED.to_owned()
        } else if is_default_export && parent.is_some() {
            INTERNAL_SYMBOL_NAME_DEFAULT.to_owned()
        } else {
            self.declaration_name(node)
        };
        let symbol = if name == INTERNAL_SYMBOL_NAME_MISSING {
            self.symbols
                .create(SymbolFlags::NONE, INTERNAL_SYMBOL_NAME_MISSING)
        } else {
            if includes.intersects(SymbolFlags::CLASSIFIABLE) {
                self.classifiable_names.insert(name.clone());
            }
            match self.table(table).and_then(|table| table.get(&name)) {
                None => {
                    let symbol = self.symbols.create(SymbolFlags::NONE, name.clone());
                    self.table_mut(table).insert(name, symbol);
                    symbol
                }
                Some(existing) if self.conflicts(existing, includes, excludes) => {
                    self.report_conflict(existing, node, includes, is_default_export);
                    self.symbols.create(SymbolFlags::NONE, name)
                }
                Some(existing) => existing,
            }
        };
        self.add_declaration_to_symbol(symbol, node, includes);
        let declared = self.symbols.symbol_mut(symbol);
        if declared.parent.is_none() {
            declared.parent = parent;
        } else {
            assert_eq!(
                declared.parent, parent,
                "an existing symbol's parent matches the new one"
            );
        }
        symbol
    }

    /// Returns whether merging `includes` into `existing` is a conflict. Assignment declarations
    /// merge with variables regardless of their other flags.
    fn conflicts(&self, existing: SymbolId, includes: SymbolFlags, excludes: SymbolFlags) -> bool {
        let flags = self.symbols.symbol(existing).flags;
        flags.intersects(excludes)
            && !(includes.intersects(SymbolFlags::VARIABLE)
                && flags.intersects(SymbolFlags::ASSIGNMENT)
                || includes.intersects(SymbolFlags::ASSIGNMENT)
                    && flags.intersects(SymbolFlags::VARIABLE))
    }

    /// Reports a declaration conflict on every earlier declaration and then on `node`.
    fn report_conflict(
        &mut self,
        existing: SymbolId,
        node: NodeId,
        includes: SymbolFlags,
        is_default_export: bool,
    ) {
        let ast = self.ast;
        let flags = self.symbols.symbol(existing).flags;
        let mut message = if flags.intersects(SymbolFlags::BLOCK_SCOPED_VARIABLE) {
            diagnostics::CANNOT_REDECLARE_BLOCK_SCOPED_VARIABLE_0
        } else {
            diagnostics::DUPLICATE_IDENTIFIER_0
        };
        let mut message_needs_name = true;
        if flags.intersects(SymbolFlags::ENUM) || includes.intersects(SymbolFlags::ENUM) {
            message = diagnostics::ENUM_DECLARATIONS_CAN_ONLY_MERGE_WITH_NAMESPACE_OR_OTHER_ENUM_DECLARATIONS;
            message_needs_name = false;
        }
        let declarations = self.symbols.symbol(existing).declarations.clone();
        // A second default export, whether a declaration or an `export default` expression,
        // reports the multiple-default-exports error instead.
        let is_export_default_expression = ast
            .node(node)
            .data()
            .as_export_assignment()
            .is_some_and(|assignment| !assignment.is_export_equals);
        let multiple_default_exports =
            !declarations.is_empty() && (is_default_export || is_export_default_expression);
        if multiple_default_exports {
            message = diagnostics::A_MODULE_CANNOT_HAVE_MULTIPLE_DEFAULT_EXPORTS;
            message_needs_name = false;
        }
        let declaration_name = ast.name_of_declaration(node).unwrap_or(node);
        let mut diagnostic =
            self.conflict_diagnostic(declaration_name, node, message, message_needs_name);
        if self.is_export_type_without_body(node, flags) {
            let name = ast
                .node(node)
                .data()
                .name()
                .and_then(|name| ast.node_text(name))
                .unwrap_or_default();
            let suggestion = format!("export type {{ {name} }}");
            diagnostic.add_related(self.create_diagnostic_for_node(
                node,
                diagnostics::DID_YOU_MEAN_0,
                &[&suggestion],
            ));
        }
        for (index, &declaration) in declarations.iter().enumerate() {
            let name_node = ast.name_of_declaration(declaration).unwrap_or(declaration);
            let mut earlier =
                self.conflict_diagnostic(name_node, declaration, message, message_needs_name);
            if multiple_default_exports {
                let related = if index == 0 {
                    diagnostics::ANOTHER_EXPORT_DEFAULT_IS_HERE
                } else {
                    diagnostics::X_AND_HERE
                };
                earlier.add_related(self.create_diagnostic_for_node(
                    declaration_name,
                    related,
                    &[],
                ));
                diagnostic.add_related(self.create_diagnostic_for_node(
                    name_node,
                    diagnostics::THE_FIRST_EXPORT_DEFAULT_IS_HERE,
                    &[],
                ));
            }
            self.add_diagnostic(earlier);
        }
        self.add_diagnostic(diagnostic);
        // An accessor conflicting with a non-accessor or the other accessor kind becomes a full
        // accessor, so every later declaration of the name is also a duplicate.
        let accessor = flags.intersects(SymbolFlags::ACCESSOR)
            && (flags & SymbolFlags::ACCESSOR) != (includes & SymbolFlags::ACCESSOR);
        if accessor {
            self.symbols.symbol_mut(existing).flags |= SymbolFlags::ACCESSOR;
        }
    }

    fn conflict_diagnostic(
        &self,
        name_node: NodeId,
        declaration: NodeId,
        message: Message,
        message_needs_name: bool,
    ) -> crate::diagnostics::Diagnostic {
        if message_needs_name {
            let display_name = self.display_name(declaration);
            self.create_diagnostic_for_node(name_node, message, &[&display_name])
        } else {
            self.create_diagnostic_for_node(name_node, message, &[])
        }
    }

    /// Returns whether `node` is `export type T;`, which may have meant `export type { T }`.
    fn is_export_type_without_body(&self, node: NodeId, existing_flags: SymbolFlags) -> bool {
        let ast = self.ast;
        ast.node(node)
            .data()
            .as_type_alias_declaration()
            .is_some_and(|alias| ast.node_is_missing(alias.type_node))
            && ast.has_syntactic_modifier(node, ModifierFlags::EXPORT)
            && existing_flags
                .intersects(SymbolFlags::ALIAS | SymbolFlags::TYPE | SymbolFlags::NAMESPACE)
    }

    fn is_default_export(&self, node: NodeId) -> bool {
        let ast = self.ast;
        ast.has_syntactic_modifier(node, ModifierFlags::DEFAULT)
            || ast
                .node(node)
                .data()
                .as_export_specifier()
                .is_some_and(|specifier| {
                    ast.node_text(specifier.name) == Some(INTERNAL_SYMBOL_NAME_DEFAULT)
                })
    }

    /// Returns the name a declaration is bound under. Computed names must be literal.
    pub(super) fn declaration_name(&self, node: NodeId) -> String {
        let ast = self.ast;
        if let Some(assignment) = ast.node(node).data().as_export_assignment() {
            return if assignment.is_export_equals {
                INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
            } else {
                INTERNAL_SYMBOL_NAME_DEFAULT
            }
            .to_owned();
        }
        if let Some(name) = ast.name_of_declaration(node) {
            if ast.is_ambient_module(node) {
                if ast.is_global_scope_augmentation(node) {
                    return INTERNAL_SYMBOL_NAME_GLOBAL.to_owned();
                }
                return format!("\"{}\"", ast.node_text(name).unwrap_or_default());
            }
            if ast.node(name).kind() == SyntaxKind::PrivateIdentifier {
                return self.private_identifier_symbol_name(node, name);
            }
            if ast.is_property_name_literal(name)
                || ast.node(name).kind() == SyntaxKind::JsxNamespacedName
            {
                return self.name_text(name);
            }
            if let Some(computed) = ast.node(name).data().as_computed_property_name() {
                let expression = computed.expression;
                if ast.is_string_or_numeric_literal_like(expression) {
                    return ast.node_text(expression).unwrap_or_default().to_owned();
                }
                if let Some(unary) = ast.node(expression).data().as_prefix_unary_expression()
                    && ast.is_signed_numeric_literal(expression)
                {
                    let operator = token_to_string(unary.operator).unwrap_or_default();
                    return format!(
                        "{operator}{}",
                        ast.node_text(unary.operand).unwrap_or_default()
                    );
                }
                unreachable!("only computed properties with literal names have declaration names");
            }
            return INTERNAL_SYMBOL_NAME_MISSING.to_owned();
        }
        match ast.node(node).kind() {
            SyntaxKind::Constructor => INTERNAL_SYMBOL_NAME_CONSTRUCTOR,
            SyntaxKind::FunctionType | SyntaxKind::CallSignature => INTERNAL_SYMBOL_NAME_CALL,
            SyntaxKind::ConstructorType | SyntaxKind::ConstructSignature => {
                INTERNAL_SYMBOL_NAME_NEW
            }
            SyntaxKind::IndexSignature => INTERNAL_SYMBOL_NAME_INDEX,
            SyntaxKind::ExportDeclaration => INTERNAL_SYMBOL_NAME_EXPORT_STAR,
            SyntaxKind::SourceFile | SyntaxKind::BinaryExpression => {
                INTERNAL_SYMBOL_NAME_EXPORT_EQUALS
            }
            _ => INTERNAL_SYMBOL_NAME_MISSING,
        }
        .to_owned()
    }

    /// Returns the text of a name node; JSX namespaced names join their parts with `:`.
    fn name_text(&self, name: NodeId) -> String {
        let ast = self.ast;
        if let NodeData::JsxNamespacedName(namespaced) = ast.node(name).data() {
            return format!(
                "{}:{}",
                ast.node_text(namespaced.namespace).unwrap_or_default(),
                ast.node_text(namespaced.name).unwrap_or_default()
            );
        }
        ast.node_text(name).unwrap_or_default().to_owned()
    }

    /// Returns the class-qualified name of a private member, `<prefix>#<class symbol>@<name>`.
    fn private_identifier_symbol_name(&self, node: NodeId, name: NodeId) -> String {
        let ast = self.ast;
        let mut current = ast.node(node).parent();
        while let Some(id) = current {
            if matches!(
                ast.node(id).kind(),
                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
            ) {
                let class = self
                    .symbol_of(id)
                    .expect("a containing class is bound before its members");
                let description = ast.node_text(name).unwrap_or_default();
                return format!(
                    "{INTERNAL_SYMBOL_NAME_PREFIX}#{}@{description}",
                    class.index()
                );
            }
            current = ast.node(id).parent();
        }
        // Without a containing class there is already a parse error.
        INTERNAL_SYMBOL_NAME_MISSING.to_owned()
    }

    /// Returns the name used in diagnostics: the declared name's source text, or `(Missing)`.
    fn display_name(&self, node: NodeId) -> String {
        if let Some(name) = self.ast.node(node).data().name() {
            return self.declaration_name_text(name).to_owned();
        }
        let name = self.declaration_name(node);
        if name == INTERNAL_SYMBOL_NAME_MISSING {
            "(Missing)".to_owned()
        } else {
            name
        }
    }

    /// Declares a module member, pairing an exported declaration's export symbol with a local
    /// symbol so locals and exports of one name conflict and references resolve locally.
    pub(super) fn declare_module_member(
        &mut self,
        node: NodeId,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> SymbolId {
        let ast = self.ast;
        let container = self.container;
        let has_export_modifier = ast
            .combined_modifier_flags(node)
            .intersects(ModifierFlags::EXPORT);
        if includes.intersects(SymbolFlags::ALIAS) {
            if ast.node(node).kind() == SyntaxKind::ExportSpecifier
                || ast.node(node).kind() == SyntaxKind::ImportEqualsDeclaration
                    && has_export_modifier
            {
                let container_symbol = self.container_symbol();
                return self.declare_symbol(
                    Table::Exports(container_symbol),
                    Some(container_symbol),
                    node,
                    includes,
                    excludes,
                );
            }
            return self.declare_symbol(Table::Locals(container), None, node, includes, excludes);
        }
        // Nested ambient modules are kept in locals so they merge only with their original
        // module as augmentations, never with other augmentations during global merging.
        if !ast.is_ambient_module(node)
            && (has_export_modifier || self.export_contexts.contains(&container))
        {
            let container_symbol = self.container_symbol();
            let is_locals_container = super::container_flags::container_flags(ast, container)
                .intersects(super::container_flags::ContainerFlags::HAS_LOCALS);
            if !is_locals_container
                || ast.has_syntactic_modifier(node, ModifierFlags::DEFAULT)
                    && self.declaration_name(node) == INTERNAL_SYMBOL_NAME_MISSING
            {
                // An unnamed default export has no local symbol.
                return self.declare_symbol(
                    Table::Exports(container_symbol),
                    Some(container_symbol),
                    node,
                    includes,
                    excludes,
                );
            }
            let export_kind = if includes.intersects(SymbolFlags::VALUE) {
                SymbolFlags::EXPORT_VALUE
            } else {
                SymbolFlags::NONE
            };
            let local =
                self.declare_symbol(Table::Locals(container), None, node, export_kind, excludes);
            let export = self.declare_symbol(
                Table::Exports(container_symbol),
                Some(container_symbol),
                node,
                includes,
                excludes,
            );
            self.symbols.symbol_mut(local).export_symbol = Some(export);
            self.local_symbols.insert(node, local);
            return local;
        }
        self.declare_symbol(Table::Locals(container), None, node, includes, excludes)
    }

    fn declare_class_member(
        &mut self,
        node: NodeId,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> SymbolId {
        let class = self.container_symbol();
        let table = if self.is_static(node) {
            Table::Exports(class)
        } else {
            Table::Members(class)
        };
        self.declare_symbol(table, Some(class), node, includes, excludes)
    }

    fn is_static(&self, node: NodeId) -> bool {
        let ast = self.ast;
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

    fn declare_source_file_member(
        &mut self,
        node: NodeId,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> SymbolId {
        if self.is_external_module() {
            self.declare_module_member(node, includes, excludes)
        } else {
            self.declare_symbol(
                Table::Locals(self.ast.root()),
                None,
                node,
                includes,
                excludes,
            )
        }
    }

    /// Declares `node` in the table its current container owns.
    pub(super) fn declare_symbol_and_add_to_symbol_table(
        &mut self,
        node: NodeId,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) -> SymbolId {
        let container = self.container;
        match self.ast.node(container).kind() {
            SyntaxKind::ModuleDeclaration => self.declare_module_member(node, includes, excludes),
            SyntaxKind::SourceFile => self.declare_source_file_member(node, includes, excludes),
            SyntaxKind::ClassExpression | SyntaxKind::ClassDeclaration => {
                self.declare_class_member(node, includes, excludes)
            }
            SyntaxKind::EnumDeclaration => {
                let symbol = self.container_symbol();
                self.declare_symbol(
                    Table::Exports(symbol),
                    Some(symbol),
                    node,
                    includes,
                    excludes,
                )
            }
            SyntaxKind::TypeLiteral
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::JsxAttributes => {
                let symbol = self.container_symbol();
                self.declare_symbol(
                    Table::Members(symbol),
                    Some(symbol),
                    node,
                    includes,
                    excludes,
                )
            }
            SyntaxKind::FunctionType
            | SyntaxKind::ConstructorType
            | SyntaxKind::CallSignature
            | SyntaxKind::ConstructSignature
            | SyntaxKind::IndexSignature
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::MappedType => {
                self.declare_symbol(Table::Locals(container), None, node, includes, excludes)
            }
            kind => unreachable!(
                "unhandled container kind {kind:?} in declare_symbol_and_add_to_symbol_table"
            ),
        }
    }

    /// Declares a `let`, `const`, class, enum, function, interface, or type alias in its block
    /// scope, or as a module member at the top level of a module.
    pub(super) fn bind_block_scoped_declaration(
        &mut self,
        node: NodeId,
        includes: SymbolFlags,
        excludes: SymbolFlags,
    ) {
        let scope = self.block_scope_container;
        match self.ast.node(scope).kind() {
            SyntaxKind::ModuleDeclaration => {
                self.declare_module_member(node, includes, excludes);
            }
            SyntaxKind::SourceFile if self.is_external_module() => {
                self.declare_module_member(node, includes, excludes);
            }
            _ => {
                self.declare_symbol(Table::Locals(scope), None, node, includes, excludes);
            }
        }
    }

    pub(super) fn add_declaration_to_symbol(
        &mut self,
        symbol: SymbolId,
        node: NodeId,
        includes: SymbolFlags,
    ) {
        let declared = self.symbols.symbol_mut(symbol);
        declared.flags |= includes;
        if !declared.declarations.contains(&node) {
            declared.declarations.push(node);
        }
        self.node_symbols.insert(node, symbol);
        // Merging a const-enum-only module with a class, function, or regular enum makes it no
        // longer const-enum-only; namespaces recalculate this themselves.
        let flags = self.symbols.symbol(symbol).flags;
        if flags.intersects(SymbolFlags::CONST_ENUM_ONLY_MODULE)
            && flags
                .intersects(SymbolFlags::FUNCTION | SymbolFlags::CLASS | SymbolFlags::REGULAR_ENUM)
        {
            self.symbols.symbol_mut(symbol).flags =
                flags.without(SymbolFlags::CONST_ENUM_ONLY_MODULE);
            self.not_const_enum_only_modules.insert(symbol);
        }
        if includes.intersects(SymbolFlags::VALUE) {
            self.set_value_declaration(symbol, node);
        }
    }

    /// Records `node` as the value declaration unless an earlier declaration takes precedence:
    /// non-assignment declarations win over assignment declarations, and non-namespace
    /// declarations win over namespaces.
    pub(super) fn set_value_declaration(&mut self, symbol: SymbolId, node: NodeId) {
        let ast = self.ast;
        let is_assignment_declaration = |id: NodeId| {
            matches!(
                ast.node(id).kind(),
                SyntaxKind::BinaryExpression
                    | SyntaxKind::PropertyAccessExpression
                    | SyntaxKind::ElementAccessExpression
                    | SyntaxKind::Identifier
                    | SyntaxKind::CallExpression
            )
        };
        let is_effective_module_declaration = |id: NodeId| {
            matches!(
                ast.node(id).kind(),
                SyntaxKind::ModuleDeclaration | SyntaxKind::Identifier
            )
        };
        let replace = self
            .symbols
            .symbol(symbol)
            .value_declaration
            .is_none_or(|current| {
                is_assignment_declaration(current) && !is_assignment_declaration(node)
                    || ast.node(current).kind() != ast.node(node).kind()
                        && is_effective_module_declaration(current)
            });
        if replace {
            self.symbols.symbol_mut(symbol).value_declaration = Some(node);
        }
    }

    fn container_symbol(&self) -> SymbolId {
        self.symbol_of(self.container)
            .expect("a container that owns members or exports is bound before its children")
    }

    fn table(&self, table: Table) -> Option<&SymbolTable> {
        match table {
            Table::Locals(container) => self.locals.get(&container),
            Table::Members(symbol) => Some(&self.symbols.symbol(symbol).members),
            Table::Exports(symbol) => Some(&self.symbols.symbol(symbol).exports),
            Table::GlobalExports => Some(&self.global_exports),
        }
    }

    fn table_mut(&mut self, table: Table) -> &mut SymbolTable {
        match table {
            Table::Locals(container) => self.locals.entry(container).or_default(),
            Table::Members(symbol) => &mut self.symbols.symbol_mut(symbol).members,
            Table::Exports(symbol) => &mut self.symbols.symbol_mut(symbol).exports,
            Table::GlobalExports => &mut self.global_exports,
        }
    }
}
