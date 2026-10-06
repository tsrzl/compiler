//! Container classification modeled on TypeScript-Go's `GetContainerFlags`.

use crate::ast::{Ast, NodeId, SyntaxKind};

/// How a node scopes the declarations and control flow inside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct ContainerFlags(u32);

#[allow(
    dead_code,
    reason = "flow-only flags are read once control flow binding is ported"
)]
impl ContainerFlags {
    /// The node neither scopes declarations nor starts control flow.
    pub(super) const NONE: Self = Self(0);
    /// The node owns a symbol table for its children, such as a class, function, or module.
    pub(super) const IS_CONTAINER: Self = Self(1 << 0);
    /// The node scopes `let`, `const`, and class declarations, such as a block or loop.
    pub(super) const IS_BLOCK_SCOPED_CONTAINER: Self = Self(1 << 1);
    /// The node starts a new control flow graph.
    pub(super) const IS_CONTROL_FLOW_CONTAINER: Self = Self(1 << 2);
    /// The node is function-like.
    pub(super) const IS_FUNCTION_LIKE: Self = Self(1 << 3);
    /// The node is a function or arrow function expression.
    pub(super) const IS_FUNCTION_EXPRESSION: Self = Self(1 << 4);
    /// The node owns a locals table.
    pub(super) const HAS_LOCALS: Self = Self(1 << 5);
    /// The node is an interface.
    pub(super) const IS_INTERFACE: Self = Self(1 << 6);
    /// The node is a method or accessor of an object literal or class expression.
    pub(super) const IS_OBJECT_LITERAL_OR_CLASS_EXPRESSION_METHOD_OR_ACCESSOR: Self = Self(1 << 7);
    /// The node binds `this`.
    pub(super) const IS_THIS_CONTAINER: Self = Self(1 << 8);
    /// Uses of `this` inside the node belong to the enclosing `this` container.
    pub(super) const PROPAGATES_THIS_KEYWORD: Self = Self(1 << 9);

    /// Returns whether any flag in `other` is set.
    pub(super) const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    pub(super) const fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// Returns how `node` scopes declarations and control flow.
pub(super) fn container_flags(ast: &Ast, node: NodeId) -> ContainerFlags {
    use ContainerFlags as F;
    let function_like = F::IS_CONTAINER
        .with(F::IS_CONTROL_FLOW_CONTAINER)
        .with(F::HAS_LOCALS)
        .with(F::IS_FUNCTION_LIKE);
    match ast.node(node).kind() {
        SyntaxKind::ClassExpression
        | SyntaxKind::ClassDeclaration
        | SyntaxKind::EnumDeclaration
        | SyntaxKind::ObjectLiteralExpression
        | SyntaxKind::TypeLiteral
        | SyntaxKind::JsxAttributes => F::IS_CONTAINER,
        SyntaxKind::InterfaceDeclaration => F::IS_CONTAINER.with(F::IS_INTERFACE),
        SyntaxKind::ModuleDeclaration
        | SyntaxKind::TypeAliasDeclaration
        | SyntaxKind::JSTypeAliasDeclaration
        | SyntaxKind::MappedType
        | SyntaxKind::IndexSignature => F::IS_CONTAINER.with(F::HAS_LOCALS),
        SyntaxKind::SourceFile => F::IS_CONTAINER
            .with(F::IS_CONTROL_FLOW_CONTAINER)
            .with(F::HAS_LOCALS),
        SyntaxKind::GetAccessor | SyntaxKind::SetAccessor | SyntaxKind::MethodDeclaration
            if is_object_literal_or_class_expression_method_or_accessor(ast, node) =>
        {
            function_like
                .with(F::IS_OBJECT_LITERAL_OR_CLASS_EXPRESSION_METHOD_OR_ACCESSOR)
                .with(F::IS_THIS_CONTAINER)
        }
        SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::Constructor
        | SyntaxKind::FunctionDeclaration
        | SyntaxKind::ClassStaticBlockDeclaration => function_like.with(F::IS_THIS_CONTAINER),
        SyntaxKind::MethodSignature
        | SyntaxKind::CallSignature
        | SyntaxKind::FunctionType
        | SyntaxKind::ConstructSignature
        | SyntaxKind::ConstructorType => function_like.with(F::PROPAGATES_THIS_KEYWORD),
        SyntaxKind::FunctionExpression => function_like
            .with(F::IS_FUNCTION_EXPRESSION)
            .with(F::IS_THIS_CONTAINER),
        SyntaxKind::ArrowFunction => function_like
            .with(F::IS_FUNCTION_EXPRESSION)
            .with(F::PROPAGATES_THIS_KEYWORD),
        SyntaxKind::ModuleBlock => F::IS_CONTROL_FLOW_CONTAINER,
        SyntaxKind::PropertyDeclaration if ast.node(node).data().initializer().is_some() => {
            F::IS_CONTROL_FLOW_CONTAINER.with(F::IS_THIS_CONTAINER)
        }
        SyntaxKind::CatchClause
        | SyntaxKind::ForStatement
        | SyntaxKind::ForInStatement
        | SyntaxKind::ForOfStatement
        | SyntaxKind::CaseBlock => F::IS_BLOCK_SCOPED_CONTAINER.with(F::HAS_LOCALS),
        SyntaxKind::Block => {
            let parent = ast
                .node(node)
                .parent()
                .map(|parent| ast.node(parent).kind());
            if parent.is_some_and(|kind| {
                is_function_like_kind(kind) || kind == SyntaxKind::ClassStaticBlockDeclaration
            }) {
                F::NONE
            } else {
                F::IS_BLOCK_SCOPED_CONTAINER.with(F::HAS_LOCALS)
            }
        }
        _ => F::NONE,
    }
}

/// Returns whether `kind` declares a function signature.
pub(super) const fn is_function_like_kind(kind: SyntaxKind) -> bool {
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

/// Returns whether a method or accessor belongs to an object literal or class expression.
pub(super) fn is_object_literal_or_class_expression_method_or_accessor(
    ast: &Ast,
    node: NodeId,
) -> bool {
    matches!(
        ast.node(node).kind(),
        SyntaxKind::MethodDeclaration | SyntaxKind::GetAccessor | SyntaxKind::SetAccessor
    ) && ast.node(node).parent().is_some_and(|parent| {
        matches!(
            ast.node(parent).kind(),
            SyntaxKind::ObjectLiteralExpression | SyntaxKind::ClassExpression
        )
    })
}
