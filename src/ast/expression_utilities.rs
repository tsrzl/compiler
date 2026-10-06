//! Expression queries modeled on TypeScript-Go's `internal/ast/utilities.go`.

use super::{Ast, NodeData, NodeFlags, NodeId, SyntaxKind};

impl Ast {
    /// Returns the operator token kind of a binary expression.
    #[must_use]
    pub fn binary_operator(&self, id: NodeId) -> Option<SyntaxKind> {
        self.node(id)
            .data()
            .as_binary_expression()
            .map(|binary| self.node(binary.operator_token).kind())
    }

    /// Returns whether the node is an access, call, or non-null expression in an optional chain.
    #[must_use]
    pub fn is_optional_chain(&self, id: NodeId) -> bool {
        let node = self.node(id);
        node.flags().intersects(NodeFlags::OPTIONAL_CHAIN)
            && matches!(
                node.kind(),
                SyntaxKind::PropertyAccessExpression
                    | SyntaxKind::ElementAccessExpression
                    | SyntaxKind::CallExpression
                    | SyntaxKind::NonNullExpression
            )
    }

    /// Returns whether the node starts an optional chain with `?.`.
    #[must_use]
    pub fn is_optional_chain_root(&self, id: NodeId) -> bool {
        self.is_optional_chain(id)
            && self.node(id).kind() != SyntaxKind::NonNullExpression
            && self.node(id).data().question_dot_token().is_some()
    }

    /// Returns whether the node ends an optional chain: its parent does not continue the chain,
    /// starts a new chain, or uses the node as an argument rather than its expression.
    #[must_use]
    pub fn is_outermost_optional_chain(&self, id: NodeId) -> bool {
        let Some(parent) = self.node(id).parent() else {
            return true;
        };
        !self.is_optional_chain(parent)
            || self.is_optional_chain_root(parent)
            || self.node(parent).data().expression() != Some(id)
    }

    /// Returns whether the node is the expression of an optional chain root.
    #[must_use]
    pub fn is_expression_of_optional_chain_root(&self, id: NodeId) -> bool {
        self.node(id).parent().is_some_and(|parent| {
            self.is_optional_chain_root(parent) && self.node(parent).data().expression() == Some(id)
        })
    }

    /// Returns whether the node is a `??` expression.
    #[must_use]
    pub fn is_nullish_coalesce(&self, id: NodeId) -> bool {
        self.binary_operator(id) == Some(SyntaxKind::QuestionQuestionToken)
    }

    /// Returns whether the node, after parentheses and `!` negations, is a `&&`, `||`, or `??`
    /// expression.
    #[must_use]
    pub fn is_logical_expression(&self, mut id: NodeId) -> bool {
        loop {
            match self.node(id).data() {
                NodeData::ParenthesizedExpression(parenthesized) => id = parenthesized.expression,
                NodeData::PrefixUnaryExpression(unary)
                    if unary.operator == SyntaxKind::ExclamationToken =>
                {
                    id = unary.operand;
                }
                _ => {
                    return self.binary_operator(id).is_some_and(|operator| {
                        matches!(
                            operator,
                            SyntaxKind::BarBarToken
                                | SyntaxKind::AmpersandAmpersandToken
                                | SyntaxKind::QuestionQuestionToken
                        )
                    });
                }
            }
        }
    }

    /// Returns whether the node is a `&&=`, `||=`, or `??=` expression.
    #[must_use]
    pub fn is_logical_or_coalescing_assignment_expression(&self, id: NodeId) -> bool {
        self.binary_operator(id)
            .is_some_and(SyntaxKind::is_logical_or_coalescing_assignment_operator)
    }

    /// Returns whether the node is written to by an assignment, update, `for-in`/`for-of`
    /// initializer, or destructuring assignment.
    #[must_use]
    pub fn is_assignment_target(&self, id: NodeId) -> bool {
        self.assignment_target(id).is_some()
    }

    /// Returns the assignment, update, or loop that writes to the node.
    #[must_use]
    pub fn assignment_target(&self, mut id: NodeId) -> Option<NodeId> {
        loop {
            let parent = self.node(id).parent()?;
            match self.node(parent).data() {
                NodeData::BinaryExpression(binary) => {
                    let operator = self.node(binary.operator_token).kind();
                    return (operator.is_assignment_operator() && binary.left == id)
                        .then_some(parent);
                }
                NodeData::PrefixUnaryExpression(unary) => {
                    return is_update_operator(unary.operator).then_some(parent);
                }
                NodeData::PostfixUnaryExpression(unary) => {
                    return is_update_operator(unary.operator).then_some(parent);
                }
                NodeData::ForInOrOfStatement(statement) => {
                    return (statement.initializer == id).then_some(parent);
                }
                NodeData::ParenthesizedExpression(_)
                | NodeData::ArrayLiteralExpression(_)
                | NodeData::SpreadElement(_)
                | NodeData::NonNullExpression(_) => id = parent,
                NodeData::SpreadAssignment(_) => id = self.node(parent).parent()?,
                NodeData::ShorthandPropertyAssignment(property) => {
                    if property.name != id {
                        return None;
                    }
                    id = self.node(parent).parent()?;
                }
                NodeData::PropertyAssignment(property) => {
                    if property.name == id {
                        return None;
                    }
                    id = self.node(parent).parent()?;
                }
                _ => return None,
            }
        }
    }

    /// Returns whether the node is an identifier, `this`, `super`, or meta-property, optionally
    /// accessed through properties and parentheses.
    #[must_use]
    pub fn is_dotted_name(&self, id: NodeId) -> bool {
        match self.node(id).data() {
            NodeData::PropertyAccessExpression(access) => self.is_dotted_name(access.expression),
            NodeData::ParenthesizedExpression(parenthesized) => {
                self.is_dotted_name(parenthesized.expression)
            }
            _ => matches!(
                self.node(id).kind(),
                SyntaxKind::Identifier
                    | SyntaxKind::ThisKeyword
                    | SyntaxKind::SuperKeyword
                    | SyntaxKind::MetaProperty
            ),
        }
    }

    /// Returns whether the node is an identifier or a property access chain of identifiers.
    #[must_use]
    pub fn is_entity_name_expression(&self, id: NodeId) -> bool {
        match self.node(id).data() {
            NodeData::Identifier(_) => true,
            NodeData::PropertyAccessExpression(access) => {
                self.node(access.name).kind() == SyntaxKind::Identifier
                    && self.is_entity_name_expression(access.expression)
            }
            _ => false,
        }
    }

    /// Returns the call that immediately invokes a function or arrow function expression.
    #[must_use]
    pub fn immediately_invoked_function_expression(&self, id: NodeId) -> Option<NodeId> {
        if !matches!(
            self.node(id).kind(),
            SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction
        ) {
            return None;
        }
        let mut previous = id;
        let mut parent = self.node(id).parent()?;
        while self.node(parent).kind() == SyntaxKind::ParenthesizedExpression {
            previous = parent;
            parent = self.node(parent).parent()?;
        }
        self.node(parent)
            .data()
            .as_call_expression()
            .is_some_and(|call| call.expression == previous)
            .then_some(parent)
    }

    /// Returns whether the node is an `=` assignment to an object or array literal pattern.
    #[must_use]
    pub fn is_destructuring_assignment(&self, id: NodeId) -> bool {
        let Some(binary) = self.node(id).data().as_binary_expression() else {
            return false;
        };
        self.node(binary.operator_token).kind() == SyntaxKind::EqualsToken
            && matches!(
                self.node(binary.left).kind(),
                SyntaxKind::ObjectLiteralExpression | SyntaxKind::ArrayLiteralExpression
            )
    }

    /// Returns whether a statement or declaration can execute code, which decides whether it is
    /// reported as unreachable.
    #[must_use]
    pub fn is_potentially_executable_node(&self, id: NodeId) -> bool {
        let kind = self.node(id).kind();
        if (SyntaxKind::FIRST_STATEMENT..=SyntaxKind::LAST_STATEMENT).contains(&kind) {
            if let NodeData::VariableStatement(statement) = self.node(id).data() {
                let list = statement.declaration_list;
                if self
                    .combined_node_flags(list)
                    .intersects(NodeFlags::BLOCK_SCOPED)
                {
                    return true;
                }
                let declarations = self
                    .node(list)
                    .data()
                    .as_variable_declaration_list()
                    .map_or(&[][..], |list| self.list(list.declarations));
                return declarations
                    .iter()
                    .any(|&declaration| self.node(declaration).data().initializer().is_some());
            }
            return true;
        }
        matches!(
            kind,
            SyntaxKind::ClassDeclaration
                | SyntaxKind::EnumDeclaration
                | SyntaxKind::ModuleDeclaration
        )
    }
}

const fn is_update_operator(operator: SyntaxKind) -> bool {
    matches!(
        operator,
        SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
    )
}
