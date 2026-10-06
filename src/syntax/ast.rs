//! Owned syntax tree nodes and their read-only accessors.

use super::TextSpan;

/// The top-level statements in a source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub(super) statements: Vec<Statement>,
}

impl Program {
    /// Returns top-level statements in source order.
    #[must_use]
    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }
}

/// A top-level statement supported by the current parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    /// A `const`, `let`, or `var` declaration.
    VariableDeclaration(VariableDeclaration),
    /// An enum declaration.
    EnumDeclaration(EnumDeclaration),
    /// A namespace declaration and its nested statements.
    NamespaceDeclaration(NamespaceDeclaration),
    /// An interface declaration.
    InterfaceDeclaration(InterfaceDeclaration),
    /// A named type alias declaration.
    TypeAliasDeclaration(TypeAliasDeclaration),
    /// A function declaration.
    FunctionDeclaration(FunctionDeclaration),
    /// A class declaration.
    ClassDeclaration(ClassDeclaration),
    /// An expression evaluated for its effects.
    ExpressionStatement(Expression),
    /// A top-level control-flow statement.
    ControlFlowStatement(FunctionBodyStatement),
    /// A top-level `break` statement.
    Break { span: TextSpan },
    /// A top-level `continue` statement.
    Continue { span: TextSpan },
    /// A module import declaration.
    ImportDeclaration(ImportDeclaration),
    /// A default export expression.
    ExportDefault(Expression),
    /// One or more named exports.
    ExportNamed(Vec<ExportSpecifier>),
    /// One or more named type-only exports.
    ExportTypeNamed(Vec<ExportSpecifier>),
    /// Named exports forwarded from another module.
    ExportNamedFrom(ExportNamedFromDeclaration),
    /// All non-default exports from another module.
    ExportAll(ExportAllDeclaration),
    /// A declaration with an `export` modifier.
    ExportedDeclaration(Box<Statement>),
}

impl Statement {
    /// Returns the variable declaration when this statement is one.
    #[must_use]
    pub const fn as_variable_declaration(&self) -> Option<&VariableDeclaration> {
        match self {
            Self::VariableDeclaration(declaration) => Some(declaration),
            Self::ExportedDeclaration(statement) => statement.as_variable_declaration(),
            Self::EnumDeclaration(_)
            | Self::NamespaceDeclaration(_)
            | Self::InterfaceDeclaration(_)
            | Self::TypeAliasDeclaration(_)
            | Self::FunctionDeclaration(_)
            | Self::ClassDeclaration(_)
            | Self::ExpressionStatement(_)
            | Self::ControlFlowStatement(_)
            | Self::Break { .. }
            | Self::Continue { .. }
            | Self::ImportDeclaration(_)
            | Self::ExportDefault(_)
            | Self::ExportNamed(_)
            | Self::ExportTypeNamed(_)
            | Self::ExportNamedFrom(_)
            | Self::ExportAll(_) => None,
        }
    }

    /// Returns the interface declaration when this statement is one.
    #[must_use]
    pub const fn as_interface_declaration(&self) -> Option<&InterfaceDeclaration> {
        match self {
            Self::InterfaceDeclaration(declaration) => Some(declaration),
            Self::ExportedDeclaration(statement) => statement.as_interface_declaration(),
            Self::VariableDeclaration(_)
            | Self::EnumDeclaration(_)
            | Self::NamespaceDeclaration(_)
            | Self::TypeAliasDeclaration(_)
            | Self::FunctionDeclaration(_)
            | Self::ClassDeclaration(_)
            | Self::ExpressionStatement(_)
            | Self::ControlFlowStatement(_)
            | Self::Break { .. }
            | Self::Continue { .. }
            | Self::ImportDeclaration(_)
            | Self::ExportDefault(_)
            | Self::ExportNamed(_)
            | Self::ExportTypeNamed(_)
            | Self::ExportNamedFrom(_)
            | Self::ExportAll(_) => None,
        }
    }

    /// Returns the type alias when this statement is one.
    #[must_use]
    pub const fn as_type_alias_declaration(&self) -> Option<&TypeAliasDeclaration> {
        match self {
            Self::TypeAliasDeclaration(declaration) => Some(declaration),
            Self::ExportedDeclaration(statement) => statement.as_type_alias_declaration(),
            Self::VariableDeclaration(_)
            | Self::EnumDeclaration(_)
            | Self::NamespaceDeclaration(_)
            | Self::InterfaceDeclaration(_)
            | Self::FunctionDeclaration(_)
            | Self::ClassDeclaration(_)
            | Self::ExpressionStatement(_)
            | Self::ControlFlowStatement(_)
            | Self::Break { .. }
            | Self::Continue { .. }
            | Self::ImportDeclaration(_)
            | Self::ExportDefault(_)
            | Self::ExportNamed(_)
            | Self::ExportTypeNamed(_)
            | Self::ExportNamedFrom(_)
            | Self::ExportAll(_) => None,
        }
    }

    /// Returns the function declaration when this statement is one.
    #[must_use]
    pub const fn as_function_declaration(&self) -> Option<&FunctionDeclaration> {
        match self {
            Self::FunctionDeclaration(declaration) => Some(declaration),
            Self::ExportedDeclaration(statement) => statement.as_function_declaration(),
            Self::VariableDeclaration(_)
            | Self::EnumDeclaration(_)
            | Self::NamespaceDeclaration(_)
            | Self::InterfaceDeclaration(_)
            | Self::TypeAliasDeclaration(_)
            | Self::ClassDeclaration(_)
            | Self::ExpressionStatement(_)
            | Self::ControlFlowStatement(_)
            | Self::Break { .. }
            | Self::Continue { .. }
            | Self::ImportDeclaration(_)
            | Self::ExportDefault(_)
            | Self::ExportNamed(_)
            | Self::ExportTypeNamed(_)
            | Self::ExportNamedFrom(_)
            | Self::ExportAll(_) => None,
        }
    }

    /// Returns the class declaration when this statement is one.
    #[must_use]
    pub const fn as_class_declaration(&self) -> Option<&ClassDeclaration> {
        match self {
            Self::ClassDeclaration(declaration) => Some(declaration),
            Self::ExportedDeclaration(statement) => statement.as_class_declaration(),
            Self::VariableDeclaration(_)
            | Self::EnumDeclaration(_)
            | Self::NamespaceDeclaration(_)
            | Self::InterfaceDeclaration(_)
            | Self::TypeAliasDeclaration(_)
            | Self::FunctionDeclaration(_)
            | Self::ExpressionStatement(_)
            | Self::ControlFlowStatement(_)
            | Self::Break { .. }
            | Self::Continue { .. }
            | Self::ImportDeclaration(_)
            | Self::ExportDefault(_)
            | Self::ExportNamed(_)
            | Self::ExportTypeNamed(_)
            | Self::ExportNamedFrom(_)
            | Self::ExportAll(_) => None,
        }
    }

    /// Returns the enum declaration when this statement is one.
    #[must_use]
    pub const fn as_enum_declaration(&self) -> Option<&EnumDeclaration> {
        match self {
            Self::EnumDeclaration(declaration) => Some(declaration),
            Self::ExportedDeclaration(statement) => statement.as_enum_declaration(),
            Self::VariableDeclaration(_)
            | Self::NamespaceDeclaration(_)
            | Self::InterfaceDeclaration(_)
            | Self::TypeAliasDeclaration(_)
            | Self::FunctionDeclaration(_)
            | Self::ClassDeclaration(_)
            | Self::ExpressionStatement(_)
            | Self::ControlFlowStatement(_)
            | Self::Break { .. }
            | Self::Continue { .. }
            | Self::ImportDeclaration(_)
            | Self::ExportDefault(_)
            | Self::ExportNamed(_)
            | Self::ExportTypeNamed(_)
            | Self::ExportNamedFrom(_)
            | Self::ExportAll(_) => None,
        }
    }

    /// Returns the namespace declaration when this statement is one.
    #[must_use]
    pub const fn as_namespace_declaration(&self) -> Option<&NamespaceDeclaration> {
        match self {
            Self::NamespaceDeclaration(declaration) => Some(declaration),
            Self::ExportedDeclaration(statement) => statement.as_namespace_declaration(),
            Self::VariableDeclaration(_)
            | Self::EnumDeclaration(_)
            | Self::InterfaceDeclaration(_)
            | Self::TypeAliasDeclaration(_)
            | Self::FunctionDeclaration(_)
            | Self::ClassDeclaration(_)
            | Self::ExpressionStatement(_)
            | Self::ControlFlowStatement(_)
            | Self::Break { .. }
            | Self::Continue { .. }
            | Self::ImportDeclaration(_)
            | Self::ExportDefault(_)
            | Self::ExportNamed(_)
            | Self::ExportTypeNamed(_)
            | Self::ExportNamedFrom(_)
            | Self::ExportAll(_) => None,
        }
    }

    /// Returns the declaration after removing export wrappers.
    #[must_use]
    pub const fn declaration(&self) -> &Self {
        match self {
            Self::ExportedDeclaration(statement) => statement.declaration(),
            _ => self,
        }
    }

    /// Returns whether this declaration has an `export` modifier.
    #[must_use]
    pub const fn is_exported(&self) -> bool {
        matches!(
            self,
            Self::ExportedDeclaration(_)
                | Self::ExportDefault(_)
                | Self::ExportNamed(_)
                | Self::ExportTypeNamed(_)
                | Self::ExportNamedFrom(_)
                | Self::ExportAll(_)
        )
    }

    /// Returns whether this statement marks the file as an external module.
    #[must_use]
    pub const fn is_external_module_indicator(&self) -> bool {
        matches!(
            self,
            Self::ExportedDeclaration(_)
                | Self::ImportDeclaration(_)
                | Self::ExportDefault(_)
                | Self::ExportNamed(_)
                | Self::ExportTypeNamed(_)
                | Self::ExportNamedFrom(_)
                | Self::ExportAll(_)
        )
    }
}

/// An import declaration with a module specifier and optional bindings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDeclaration {
    pub(super) module_specifier: String,
    pub(super) raw_module_specifier: String,
    pub(super) type_only: bool,
    pub(super) default_import: Option<ImportSpecifier>,
    pub(super) namespace_import: Option<ImportSpecifier>,
    pub(super) named_imports: Vec<ImportSpecifier>,
    pub(super) span: TextSpan,
}

/// A named import and the local binding it introduces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSpecifier {
    pub(super) imported_name: String,
    pub(super) local_name: String,
    pub(super) span: TextSpan,
}

impl ImportSpecifier {
    /// Returns the exported name requested from the module.
    #[must_use]
    pub fn imported_name(&self) -> &str {
        &self.imported_name
    }

    /// Returns the local name introduced by the import.
    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }

    /// Returns the source span of the imported name.
    #[must_use]
    pub const fn span(&self) -> TextSpan {
        self.span
    }
}

/// A local name and its public name in a named export list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportSpecifier {
    pub(super) local_name: String,
    pub(super) exported_name: String,
    pub(super) span: TextSpan,
}

/// A named export list forwarded from another module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportNamedFromDeclaration {
    pub(super) specifiers: Vec<ExportSpecifier>,
    pub(super) module_specifier: String,
    pub(super) raw_module_specifier: String,
    pub(super) span: TextSpan,
    pub(super) type_only: bool,
}

impl ExportNamedFromDeclaration {
    /// Returns the source names and their public aliases in source order.
    #[must_use]
    pub fn specifiers(&self) -> &[ExportSpecifier] {
        &self.specifiers
    }

    /// Returns the unquoted module specifier.
    #[must_use]
    pub fn module_specifier(&self) -> &str {
        &self.module_specifier
    }

    /// Returns the original module specifier including its quote delimiters.
    #[must_use]
    pub fn raw_module_specifier(&self) -> &str {
        &self.raw_module_specifier
    }

    /// Returns the source span of the quoted module specifier.
    #[must_use]
    pub const fn span(&self) -> TextSpan {
        self.span
    }

    /// Returns whether the export is restricted to the type namespace.
    #[must_use]
    pub const fn is_type_only(&self) -> bool {
        self.type_only
    }
}

/// An `export * from` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportAllDeclaration {
    pub(super) module_specifier: String,
    pub(super) raw_module_specifier: String,
    pub(super) span: TextSpan,
}

impl ExportAllDeclaration {
    /// Returns the unquoted module specifier.
    #[must_use]
    pub fn module_specifier(&self) -> &str {
        &self.module_specifier
    }

    /// Returns the original module specifier including its quote delimiters.
    #[must_use]
    pub fn raw_module_specifier(&self) -> &str {
        &self.raw_module_specifier
    }

    /// Returns the source span of the quoted module specifier.
    #[must_use]
    pub const fn span(&self) -> TextSpan {
        self.span
    }
}

impl ExportSpecifier {
    /// Returns the name bound in the current source file.
    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }

    /// Returns the name exposed by the module.
    #[must_use]
    pub fn exported_name(&self) -> &str {
        &self.exported_name
    }

    /// Returns the source span of the local name.
    #[must_use]
    pub const fn span(&self) -> TextSpan {
        self.span
    }
}

impl ImportDeclaration {
    /// Returns the unquoted module specifier.
    #[must_use]
    pub fn module_specifier(&self) -> &str {
        &self.module_specifier
    }

    /// Returns the module specifier with its original quote delimiters.
    #[must_use]
    pub fn raw_module_specifier(&self) -> &str {
        &self.raw_module_specifier
    }

    /// Returns whether the import is restricted to the type namespace.
    #[must_use]
    pub const fn is_type_only(&self) -> bool {
        self.type_only
    }

    /// Returns the optional default import and its local binding.
    #[must_use]
    pub fn default_import(&self) -> Option<&ImportSpecifier> {
        self.default_import.as_ref()
    }

    /// Returns the optional namespace import and its local binding.
    #[must_use]
    pub fn namespace_import(&self) -> Option<&ImportSpecifier> {
        self.namespace_import.as_ref()
    }

    /// Returns named imports in source order.
    #[must_use]
    pub fn named_imports(&self) -> &[ImportSpecifier] {
        &self.named_imports
    }

    /// Returns default and named imports in source order.
    pub fn imported_bindings(&self) -> impl Iterator<Item = &ImportSpecifier> {
        self.default_import
            .iter()
            .chain(self.namespace_import.iter())
            .chain(self.named_imports.iter())
    }

    /// Returns the source span of the quoted module specifier.
    #[must_use]
    pub const fn span(&self) -> TextSpan {
        self.span
    }
}

/// A function declaration with typed parameters and a function body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDeclaration {
    pub(super) name: String,
    pub(super) accessibility: Option<MemberAccessibility>,
    pub(super) is_static: bool,
    pub(super) parameters: Vec<FunctionParameter>,
    pub(super) return_type: Option<TypeReference>,
    pub(super) body: Vec<FunctionBodyStatement>,
}

impl FunctionDeclaration {
    /// Returns the function name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns whether the method is static.
    #[must_use]
    pub const fn is_static(&self) -> bool {
        self.is_static
    }

    /// Returns whether the method is private.
    #[must_use]
    pub const fn is_private(&self) -> bool {
        matches!(self.accessibility, Some(MemberAccessibility::Private))
    }

    /// Returns whether the method is protected.
    #[must_use]
    pub const fn is_protected(&self) -> bool {
        matches!(self.accessibility, Some(MemberAccessibility::Protected))
    }

    /// Returns parameters in declaration order.
    #[must_use]
    pub fn parameters(&self) -> &[FunctionParameter] {
        &self.parameters
    }

    /// Returns the optional return type annotation.
    #[must_use]
    pub const fn return_type(&self) -> Option<&TypeReference> {
        self.return_type.as_ref()
    }

    /// Returns statements in body order.
    #[must_use]
    pub fn body(&self) -> &[FunctionBodyStatement] {
        &self.body
    }

    /// Returns explicit return expressions in source order.
    #[must_use]
    pub fn return_expressions(&self) -> Vec<Option<&Expression>> {
        let mut expressions = Vec::new();
        collect_return_expressions(&self.body, &mut expressions);
        expressions
    }
}

/// A class declaration with its named methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassDeclaration {
    pub(super) name: String,
    pub(super) base_class: Option<String>,
    pub(super) base_class_span: Option<TextSpan>,
    pub(super) members: Vec<ClassMember>,
}

impl ClassDeclaration {
    /// Returns the class name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the simple base class name, when the declaration has an `extends` clause.
    #[must_use]
    pub fn base_class(&self) -> Option<&str> {
        self.base_class.as_deref()
    }

    /// Returns the source span of the base class name.
    #[must_use]
    pub const fn base_class_span(&self) -> Option<TextSpan> {
        self.base_class_span
    }

    /// Returns class members in source order.
    #[must_use]
    pub fn members(&self) -> &[ClassMember] {
        &self.members
    }
}

/// A TypeScript enum with its members in declaration order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDeclaration {
    pub(super) name: String,
    pub(super) members: Vec<EnumMember>,
    pub(super) is_const: bool,
}

impl EnumDeclaration {
    /// Returns the enum name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the enum members in declaration order.
    #[must_use]
    pub fn members(&self) -> &[EnumMember] {
        &self.members
    }

    /// Returns whether the enum is a compile-time constant enum.
    #[must_use]
    pub const fn is_const(&self) -> bool {
        self.is_const
    }
}

/// A namespace declaration containing its owned nested statements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceDeclaration {
    pub(super) name: String,
    pub(super) name_span: TextSpan,
    pub(super) members: Vec<Statement>,
    pub(super) span: TextSpan,
}

impl NamespaceDeclaration {
    /// Returns the namespace name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the source span of the namespace name.
    #[must_use]
    pub const fn name_span(&self) -> TextSpan {
        self.name_span
    }

    /// Returns nested statements in source order.
    #[must_use]
    pub fn members(&self) -> &[Statement] {
        &self.members
    }

    /// Returns the full source span of this declaration.
    #[must_use]
    pub const fn span(&self) -> TextSpan {
        self.span
    }
}

/// A member in an enum declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumMember {
    pub(super) name: String,
    pub(super) name_span: TextSpan,
    pub(super) initializer: Option<Expression>,
}

impl EnumMember {
    /// Returns the member name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the source span of the member name.
    #[must_use]
    pub const fn name_span(&self) -> TextSpan {
        self.name_span
    }

    /// Returns the optional member initializer.
    #[must_use]
    pub const fn initializer(&self) -> Option<&Expression> {
        self.initializer.as_ref()
    }
}

/// A class member supported by the syntax tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassMember {
    /// An instance method.
    Method(FunctionDeclaration),
    /// An instance property.
    Property(PropertyDeclaration),
}

impl ClassMember {
    /// Returns whether this member belongs to the class constructor rather than its instances.
    #[must_use]
    pub const fn is_static(&self) -> bool {
        match self {
            Self::Method(method) => method.is_static(),
            Self::Property(property) => property.is_static(),
        }
    }

    /// Returns the method held by this member.
    #[must_use]
    pub const fn as_method(&self) -> Option<&FunctionDeclaration> {
        match self {
            Self::Method(method) => Some(method),
            Self::Property(_) => None,
        }
    }

    /// Returns the property held by this member.
    #[must_use]
    pub const fn as_property(&self) -> Option<&PropertyDeclaration> {
        match self {
            Self::Property(property) => Some(property),
            Self::Method(_) => None,
        }
    }
}

/// A class instance property with an optional type annotation and initializer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyDeclaration {
    pub(super) name: String,
    pub(super) name_span: TextSpan,
    pub(super) accessibility: Option<MemberAccessibility>,
    pub(super) is_static: bool,
    pub(super) readonly: bool,
    pub(super) type_annotation: Option<TypeReference>,
    pub(super) initializer: Option<Expression>,
}

impl PropertyDeclaration {
    /// Returns the property name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns whether the property is static.
    #[must_use]
    pub const fn is_static(&self) -> bool {
        self.is_static
    }

    /// Returns the source span of the property name.
    #[must_use]
    pub const fn name_span(&self) -> TextSpan {
        self.name_span
    }

    /// Returns whether the property is private.
    #[must_use]
    pub const fn is_private(&self) -> bool {
        matches!(self.accessibility, Some(MemberAccessibility::Private))
    }

    /// Returns whether the property is protected.
    #[must_use]
    pub const fn is_protected(&self) -> bool {
        matches!(self.accessibility, Some(MemberAccessibility::Protected))
    }

    /// Returns whether the property is readonly.
    #[must_use]
    pub const fn is_readonly(&self) -> bool {
        self.readonly
    }

    /// Returns the optional property type annotation.
    #[must_use]
    pub const fn type_annotation(&self) -> Option<&TypeReference> {
        self.type_annotation.as_ref()
    }

    /// Returns the optional property initializer.
    #[must_use]
    pub const fn initializer(&self) -> Option<&Expression> {
        self.initializer.as_ref()
    }
}

/// A statement in a function body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionBodyStatement {
    /// A local variable declaration.
    VariableDeclaration(VariableDeclaration),
    /// An expression evaluated for its effects.
    Expression(Expression),
    /// A function return statement.
    Return(ReturnStatement),
    /// A statement that throws the evaluated expression.
    Throw(Expression),
    /// A conditional statement and its optional alternative branch.
    If {
        /// The condition evaluated before selecting a branch.
        condition: Expression,
        /// Statements executed when the condition is truthy.
        then_body: Vec<FunctionBodyStatement>,
        /// Statements executed when the condition is falsy.
        else_body: Option<Vec<FunctionBodyStatement>>,
    },
    /// A loop that repeats while its condition is truthy.
    While {
        /// The condition evaluated before each iteration.
        condition: Expression,
        /// Statements executed for each iteration.
        body: Vec<FunctionBodyStatement>,
    },
    /// A loop that executes its body before checking whether to repeat.
    DoWhile {
        /// Statements executed before each condition check.
        body: Vec<FunctionBodyStatement>,
        /// The condition evaluated after each iteration.
        condition: Expression,
    },
    /// A loop with optional initializer, condition, and incrementor clauses.
    For {
        /// The declaration or expression evaluated before the first condition check.
        initializer: Option<ForInitializer>,
        /// The condition evaluated before each iteration.
        condition: Option<Expression>,
        /// The expression evaluated after each iteration.
        incrementor: Option<Expression>,
        /// Statements executed for each iteration.
        body: Vec<FunctionBodyStatement>,
    },
    /// A loop that iterates over values produced by an iterable expression.
    ForOf {
        /// The declaration or assignable expression receiving each value.
        initializer: ForInitializer,
        /// The iterable evaluated once before the first iteration.
        iterable: Expression,
        /// Statements executed for each value.
        body: Vec<FunctionBodyStatement>,
    },
    /// A loop that visits enumerable property names of an object expression.
    ForIn {
        /// The declaration or assignable expression receiving each property name.
        initializer: ForInitializer,
        /// The object whose enumerable properties are visited.
        object: Expression,
        /// Statements executed for each property name.
        body: Vec<FunctionBodyStatement>,
    },
    /// A branch statement selected by matching a value against case labels.
    Switch {
        /// The value evaluated once to select a clause.
        expression: Expression,
        /// Clauses in source order, including an optional default clause.
        clauses: Vec<SwitchClause>,
    },
    /// A protected statement with optional exception and cleanup clauses.
    Try {
        /// Statements executed inside the protected block.
        try_body: Vec<FunctionBodyStatement>,
        /// The optional exception binding and handler body.
        catch_clause: Option<CatchClause>,
        /// Statements executed during cleanup.
        finally_body: Option<Vec<FunctionBodyStatement>>,
    },
    /// A `break` statement.
    Break { span: TextSpan },
    /// A `continue` statement.
    Continue { span: TextSpan },
}

/// The expression or statement block executed by an arrow function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArrowFunctionBody {
    /// An expression returned by the function.
    Expression(Box<Expression>),
    /// Statements executed by the function.
    Block(Vec<FunctionBodyStatement>),
}

impl FunctionBodyStatement {
    /// Returns the returned expression, when this is a return statement.
    #[must_use]
    pub const fn expression(&self) -> Option<&Expression> {
        match self {
            Self::Return(statement) => statement.expression(),
            Self::VariableDeclaration(_)
            | Self::Expression(_)
            | Self::If { .. }
            | Self::Throw(_)
            | Self::While { .. }
            | Self::DoWhile { .. }
            | Self::For { .. }
            | Self::ForOf { .. }
            | Self::ForIn { .. }
            | Self::Switch { .. }
            | Self::Try { .. }
            | Self::Break { .. }
            | Self::Continue { .. } => None,
        }
    }
}

/// The optional binding and statements in a `catch` clause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatchClause {
    pub(super) variable: Option<VariableDeclaration>,
    pub(super) body: Vec<FunctionBodyStatement>,
}

impl CatchClause {
    /// Returns the optional exception binding.
    #[must_use]
    pub const fn variable(&self) -> Option<&VariableDeclaration> {
        self.variable.as_ref()
    }

    /// Returns the handler statements.
    #[must_use]
    pub fn body(&self) -> &[FunctionBodyStatement] {
        &self.body
    }
}

/// One case or default branch within a switch statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchClause {
    pub(super) expression: Option<Expression>,
    pub(super) statements: Vec<FunctionBodyStatement>,
}

impl SwitchClause {
    /// Returns the case expression, or `None` for a default clause.
    #[must_use]
    pub const fn expression(&self) -> Option<&Expression> {
        self.expression.as_ref()
    }

    /// Returns the statements in this clause.
    #[must_use]
    pub fn statements(&self) -> &[FunctionBodyStatement] {
        &self.statements
    }
}

impl ArrowFunctionBody {
    /// Returns explicit return expressions in source order.
    #[must_use]
    pub fn return_expressions(&self) -> Vec<Option<&Expression>> {
        let mut expressions = Vec::new();
        match self {
            Self::Expression(expression) => expressions.push(Some(expression.as_ref())),
            Self::Block(statements) => collect_return_expressions(statements, &mut expressions),
        }
        expressions
    }
}

fn collect_return_expressions<'body>(
    statements: &'body [FunctionBodyStatement],
    expressions: &mut Vec<Option<&'body Expression>>,
) {
    for statement in statements {
        match statement {
            FunctionBodyStatement::Return(return_statement) => {
                expressions.push(return_statement.expression());
            }
            FunctionBodyStatement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_return_expressions(then_body, expressions);
                if let Some(else_body) = else_body {
                    collect_return_expressions(else_body, expressions);
                }
            }
            FunctionBodyStatement::While { body, .. }
            | FunctionBodyStatement::DoWhile { body, .. }
            | FunctionBodyStatement::For { body, .. }
            | FunctionBodyStatement::ForOf { body, .. }
            | FunctionBodyStatement::ForIn { body, .. } => {
                collect_return_expressions(body, expressions);
            }
            FunctionBodyStatement::Switch { clauses, .. } => {
                for clause in clauses {
                    collect_return_expressions(clause.statements(), expressions);
                }
            }
            FunctionBodyStatement::Try {
                try_body,
                catch_clause,
                finally_body,
            } => {
                collect_return_expressions(try_body, expressions);
                if let Some(catch_clause) = catch_clause {
                    collect_return_expressions(catch_clause.body(), expressions);
                }
                if let Some(finally_body) = finally_body {
                    collect_return_expressions(finally_body, expressions);
                }
            }
            FunctionBodyStatement::VariableDeclaration(_)
            | FunctionBodyStatement::Expression(_)
            | FunctionBodyStatement::Throw(_)
            | FunctionBodyStatement::Break { .. }
            | FunctionBodyStatement::Continue { .. } => {}
        }
    }
}

/// The initializer clause of a C-style `for` loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForInitializer {
    /// Variable declarations such as `let index = 0, total = 0`.
    VariableDeclarations(Vec<VariableDeclaration>),
    /// An expression evaluated before the first condition check.
    Expression(Expression),
}

/// The accessibility modifier applied to a class member.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MemberAccessibility {
    Public,
    Private,
    Protected,
}

/// A function parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionParameter {
    pub(super) name: String,
    pub(super) optional: bool,
    pub(super) type_annotation: Option<TypeReference>,
    pub(super) initializer: Option<Expression>,
    pub(super) parameter_property_accessibility: Option<MemberAccessibility>,
    pub(super) parameter_property_readonly: bool,
    pub(super) span: TextSpan,
}

impl FunctionParameter {
    /// Returns the parameter name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns whether the parameter is optional.
    #[must_use]
    pub const fn is_optional(&self) -> bool {
        self.optional || self.initializer.is_some()
    }

    /// Returns the optional parameter type annotation.
    #[must_use]
    pub const fn type_annotation(&self) -> Option<&TypeReference> {
        self.type_annotation.as_ref()
    }

    /// Returns the optional default value expression.
    #[must_use]
    pub const fn initializer(&self) -> Option<&Expression> {
        self.initializer.as_ref()
    }

    /// Returns whether a constructor parameter also declares an instance property.
    #[must_use]
    pub const fn is_parameter_property(&self) -> bool {
        self.parameter_property_accessibility.is_some() || self.parameter_property_readonly
    }

    /// Returns whether this parameter property has private accessibility.
    #[must_use]
    pub const fn is_private_parameter_property(&self) -> bool {
        matches!(
            self.parameter_property_accessibility,
            Some(MemberAccessibility::Private)
        )
    }

    /// Returns whether this parameter property has protected accessibility.
    #[must_use]
    pub const fn is_protected_parameter_property(&self) -> bool {
        matches!(
            self.parameter_property_accessibility,
            Some(MemberAccessibility::Protected)
        )
    }

    /// Returns whether this parameter property is readonly.
    #[must_use]
    pub const fn is_readonly_parameter_property(&self) -> bool {
        self.parameter_property_readonly
    }

    /// Returns the source span of the parameter name.
    #[must_use]
    pub const fn span(&self) -> TextSpan {
        self.span
    }
}

/// A return statement in a function body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnStatement {
    pub(super) expression: Option<Expression>,
    pub(super) span: TextSpan,
}

impl ReturnStatement {
    /// Returns the optional returned expression.
    #[must_use]
    pub const fn expression(&self) -> Option<&Expression> {
        self.expression.as_ref()
    }

    /// Returns the source range of the `return` keyword.
    #[must_use]
    pub const fn span(&self) -> TextSpan {
        self.span
    }
}

/// A named alias for another type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAliasDeclaration {
    pub(super) name: String,
    pub(super) type_annotation: TypeReference,
}

impl TypeAliasDeclaration {
    /// Returns the alias name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the aliased type.
    #[must_use]
    pub const fn type_annotation(&self) -> &TypeReference {
        &self.type_annotation
    }
}

/// A parsed TypeScript interface declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceDeclaration {
    pub(super) name: String,
    pub(super) members: Vec<PropertySignature>,
}

impl InterfaceDeclaration {
    /// Returns the interface name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns property signatures in declaration order.
    #[must_use]
    pub fn members(&self) -> &[PropertySignature] {
        &self.members
    }
}

/// A property signature declared by an interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertySignature {
    pub(super) name: String,
    pub(super) optional: bool,
    pub(super) type_annotation: TypeReference,
}

impl PropertySignature {
    /// Returns the property name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns whether the property is optional.
    #[must_use]
    pub const fn is_optional(&self) -> bool {
        self.optional
    }

    /// Returns the property's type annotation.
    #[must_use]
    pub const fn type_annotation(&self) -> &TypeReference {
        &self.type_annotation
    }
}

/// A parsed variable declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableDeclaration {
    pub(super) declaration_kind: VariableDeclarationKind,
    pub(super) name: String,
    pub(super) name_span: TextSpan,
    pub(super) type_annotation: Option<TypeReference>,
    pub(super) initializer: Option<Expression>,
}

impl VariableDeclaration {
    /// Returns whether the declaration used `const`, `let`, or `var`.
    #[must_use]
    pub const fn declaration_kind(&self) -> VariableDeclarationKind {
        self.declaration_kind
    }

    /// Returns the declared binding name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the source span of the binding name.
    #[must_use]
    pub const fn name_span(&self) -> TextSpan {
        self.name_span
    }

    /// Returns the optional type annotation.
    #[must_use]
    pub const fn type_annotation(&self) -> Option<&TypeReference> {
        self.type_annotation.as_ref()
    }

    /// Returns the optional initializer expression.
    #[must_use]
    pub const fn initializer(&self) -> Option<&Expression> {
        self.initializer.as_ref()
    }
}

/// The declaration keyword used by a variable declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariableDeclarationKind {
    /// An immutable binding.
    Const,
    /// A block-scoped mutable binding.
    Let,
    /// A function-scoped mutable binding.
    Var,
}

/// A type annotation represented by one or more named types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeReference {
    pub(super) names: Vec<String>,
    pub(super) array_dimensions: Vec<usize>,
    pub(super) predicate_parameter: Option<String>,
    pub(super) span: TextSpan,
}

impl TypeReference {
    /// Returns the referenced type name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.names[0]
    }

    /// Returns the named types in a union annotation, or its single named type.
    #[must_use]
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// Returns each union member's source-level type spelling.
    pub fn type_spellings(&self) -> impl Iterator<Item = String> + '_ {
        self.names
            .iter()
            .zip(&self.array_dimensions)
            .map(|(name, dimensions)| format!("{name}{}", "[]".repeat(*dimensions)))
    }

    /// Returns the parameter named by a type-predicate return annotation.
    #[must_use]
    pub fn predicate_parameter(&self) -> Option<&str> {
        self.predicate_parameter.as_deref()
    }

    /// Returns the source-level spelling used when emitting a function return type.
    #[must_use]
    pub fn return_type_spelling(&self) -> String {
        let type_spelling = self.type_spellings().collect::<Vec<_>>().join(" | ");
        match self.predicate_parameter.as_deref() {
            Some(name) => format!("{name} is {type_spelling}"),
            None => type_spelling,
        }
    }

    /// Returns the source span of the type annotation.
    #[must_use]
    pub const fn span(&self) -> TextSpan {
        self.span
    }
}

/// An expression supported by the current parser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    /// A numeric literal, retained in source spelling.
    NumberLiteral {
        /// The literal's source spelling.
        value: String,
        /// The literal's source span.
        span: TextSpan,
    },
    /// A boolean literal.
    BooleanLiteral {
        /// The literal value.
        value: bool,
        /// The literal's source span.
        span: TextSpan,
    },
    /// The null literal.
    NullLiteral {
        /// The literal's source span.
        span: TextSpan,
    },
    /// A string or template literal, retained in its source spelling.
    StringLiteral {
        /// The literal's source spelling, including its delimiters.
        raw: String,
        /// The literal's source span.
        span: TextSpan,
    },
    /// An identifier expression.
    Identifier {
        /// The identifier name.
        name: String,
        /// The identifier's source span.
        span: TextSpan,
    },
    /// An addition or subtraction expression.
    BinaryExpression {
        /// The left operand.
        left: Box<Expression>,
        /// The binary operator.
        operator: BinaryOperator,
        /// The operator's source span.
        operator_span: TextSpan,
        /// The right operand.
        right: Box<Expression>,
        /// The full expression source span.
        span: TextSpan,
    },
    /// A conditional expression with a condition and two result branches.
    ConditionalExpression {
        /// The expression whose truthiness selects a branch.
        condition: Box<Expression>,
        /// The result when the condition is truthy.
        when_true: Box<Expression>,
        /// The result when the condition is falsy.
        when_false: Box<Expression>,
        /// The full expression source span.
        span: TextSpan,
    },
    /// Assignment of a value to an expression target.
    AssignmentExpression {
        /// The variable or property being assigned.
        left: Box<Expression>,
        /// The assigned value.
        right: Box<Expression>,
        /// The assignment operator.
        operator: AssignmentOperator,
        /// The assignment operator's source span.
        operator_span: TextSpan,
        /// The full expression source span.
        span: TextSpan,
    },
    /// An expression surrounded by parentheses.
    ParenthesizedExpression {
        /// The grouped expression.
        expression: Box<Expression>,
        /// The full expression source span, including parentheses.
        span: TextSpan,
    },
    /// An expression with a compile-time-only type assertion.
    TypeAssertionExpression {
        /// The expression whose type is asserted.
        expression: Box<Expression>,
        /// The asserted type.
        type_annotation: TypeReference,
        /// The full expression source span.
        span: TextSpan,
    },
    /// A call expression with positional arguments.
    CallExpression {
        /// The expression being called.
        callee: Box<Expression>,
        /// Arguments in source order.
        arguments: Vec<Expression>,
        /// The full call source span.
        span: TextSpan,
    },
    /// Construction of a value using a constructor expression.
    NewExpression {
        /// The class or constructor being instantiated.
        constructor: Box<Expression>,
        /// Arguments in source order.
        arguments: Vec<Expression>,
        /// The full construction source span.
        span: TextSpan,
    },
    /// An expression-bodied arrow function.
    ArrowFunction {
        /// Parameters in declaration order.
        parameters: Vec<FunctionParameter>,
        /// Whether the source wrapped the parameter list in parentheses.
        parameters_parenthesized: bool,
        /// The optional result type annotation.
        return_type: Option<TypeReference>,
        /// The expression or statements executed by the function.
        body: ArrowFunctionBody,
        /// The full arrow function source span.
        span: TextSpan,
    },
    /// Access to a named property on another expression.
    PropertyAccessExpression {
        /// The value whose property is accessed.
        receiver: Box<Expression>,
        /// The property name.
        name: String,
        /// The property name source span.
        name_span: TextSpan,
        /// The full access source span.
        span: TextSpan,
    },
    /// Access to an array or tuple element by index.
    ElementAccessExpression {
        /// The value whose element is accessed.
        receiver: Box<Expression>,
        /// The element index expression.
        argument: Box<Expression>,
        /// The full access source span.
        span: TextSpan,
    },
    /// A prefix arithmetic operation.
    UnaryExpression {
        /// The prefix operator.
        operator: UnaryOperator,
        /// The operand expression.
        operand: Box<Expression>,
        /// The full expression source span.
        span: TextSpan,
    },
    /// An object literal with named property assignments.
    ObjectLiteral {
        /// The object's property assignments.
        properties: Vec<ObjectProperty>,
        /// The full object literal source span.
        span: TextSpan,
    },
    /// An array literal with positional elements.
    ArrayLiteral {
        /// Elements in source order.
        elements: Vec<Expression>,
        /// The full array literal source span.
        span: TextSpan,
    },
}

/// An assignment operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssignmentOperator {
    /// Replaces the target value with the right-hand value.
    Assign,
    /// Adds the right-hand value to the target value.
    AddAssign,
}

impl AssignmentOperator {
    /// Returns the operator's source spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Assign => "=",
            Self::AddAssign => "+=",
        }
    }
}

/// A named property assignment in an object literal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectProperty {
    pub(super) name: String,
    pub(super) name_span: TextSpan,
    pub(super) value: Expression,
}

impl ObjectProperty {
    /// Returns the property name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the source span of the property name.
    #[must_use]
    pub const fn name_span(&self) -> TextSpan {
        self.name_span
    }

    /// Returns the assigned expression.
    #[must_use]
    pub const fn value(&self) -> &Expression {
        &self.value
    }
}

/// A binary expression operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOperator {
    /// Numeric addition or string concatenation.
    Add,
    /// Numeric subtraction.
    Subtract,
    /// Numeric multiplication.
    Multiply,
    /// Numeric division.
    Divide,
    /// Numeric remainder.
    Remainder,
    /// Numeric or string less-than comparison.
    LessThan,
    /// Numeric or string greater-than comparison.
    GreaterThan,
    /// Numeric or string less-than-or-equal comparison.
    LessThanOrEqual,
    /// Numeric or string greater-than-or-equal comparison.
    GreaterThanOrEqual,
    /// Equality comparison.
    Equal,
    /// Strict equality comparison.
    StrictEqual,
    /// Inequality comparison.
    NotEqual,
    /// Strict inequality comparison.
    StrictNotEqual,
    /// Logical conjunction with short-circuit evaluation.
    LogicalAnd,
    /// Logical disjunction with short-circuit evaluation.
    LogicalOr,
    /// Nullish coalescing.
    NullishCoalesce,
}

/// A prefix arithmetic operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOperator {
    /// Unary plus.
    Plus,
    /// Unary negation.
    Negate,
    /// Logical negation.
    LogicalNot,
}

impl UnaryOperator {
    /// Returns the source spelling of the operator.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Plus => "+",
            Self::Negate => "-",
            Self::LogicalNot => "!",
        }
    }
}

impl BinaryOperator {
    /// Returns the source spelling of the operator.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Remainder => "%",
            Self::LessThan => "<",
            Self::GreaterThan => ">",
            Self::LessThanOrEqual => "<=",
            Self::GreaterThanOrEqual => ">=",
            Self::Equal => "==",
            Self::StrictEqual => "===",
            Self::NotEqual => "!=",
            Self::StrictNotEqual => "!==",
            Self::LogicalAnd => "&&",
            Self::LogicalOr => "||",
            Self::NullishCoalesce => "??",
        }
    }
}

impl Expression {
    /// Returns the source span of the expression.
    #[must_use]
    pub const fn span(&self) -> TextSpan {
        match self {
            Self::NumberLiteral { span, .. }
            | Self::BooleanLiteral { span, .. }
            | Self::NullLiteral { span }
            | Self::StringLiteral { span, .. }
            | Self::Identifier { span, .. }
            | Self::BinaryExpression { span, .. }
            | Self::ConditionalExpression { span, .. }
            | Self::AssignmentExpression { span, .. }
            | Self::ParenthesizedExpression { span, .. }
            | Self::TypeAssertionExpression { span, .. }
            | Self::CallExpression { span, .. }
            | Self::NewExpression { span, .. }
            | Self::ArrowFunction { span, .. }
            | Self::PropertyAccessExpression { span, .. }
            | Self::ElementAccessExpression { span, .. }
            | Self::UnaryExpression { span, .. }
            | Self::ObjectLiteral { span, .. }
            | Self::ArrayLiteral { span, .. } => *span,
        }
    }
}
