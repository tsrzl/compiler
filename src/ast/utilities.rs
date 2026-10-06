//! Declaration and modifier queries modeled on TypeScript-Go's `internal/ast/utilities.go`.

use super::{Ast, ModifierFlags, NodeData, NodeFlags, NodeId, SyntaxKind};

impl Ast {
    /// Returns the flags of the node's own modifiers.
    #[must_use]
    pub fn modifier_flags(&self, id: NodeId) -> ModifierFlags {
        self.node(id)
            .data()
            .modifiers()
            .map_or(ModifierFlags::NONE, |modifiers| modifiers.flags())
    }

    /// Returns whether the node's own modifiers include any of `flags`.
    #[must_use]
    pub fn has_syntactic_modifier(&self, id: NodeId, flags: ModifierFlags) -> bool {
        self.modifier_flags(id).intersects(flags)
    }

    /// Returns the declaration that owns a binding element, walking out of binding patterns.
    #[must_use]
    pub fn root_declaration(&self, mut id: NodeId) -> NodeId {
        while self.node(id).kind() == SyntaxKind::BindingElement {
            id = self.grandparent(id);
        }
        id
    }

    /// Returns the node flags combined with those of its variable declaration list and statement.
    #[must_use]
    pub fn combined_node_flags(&self, id: NodeId) -> NodeFlags {
        let mut flags = NodeFlags::NONE;
        self.for_each_combined_declaration(id, |node| flags |= self.node(node).flags());
        flags
    }

    /// Returns the modifier flags combined with those of its variable declaration list and statement.
    #[must_use]
    pub fn combined_modifier_flags(&self, id: NodeId) -> ModifierFlags {
        let mut flags = ModifierFlags::NONE;
        self.for_each_combined_declaration(id, |node| flags |= self.modifier_flags(node));
        flags
    }

    /// Returns whether a declaration is `let`, `const`, `using`, or a `catch` clause variable.
    #[must_use]
    pub fn is_block_or_catch_scoped(&self, id: NodeId) -> bool {
        self.combined_node_flags(id)
            .intersects(NodeFlags::BLOCK_SCOPED)
            || self.is_catch_clause_variable_declaration_or_binding_element(id)
    }

    /// Returns whether the declaration is, or is destructured from, a `catch` clause variable.
    #[must_use]
    pub fn is_catch_clause_variable_declaration_or_binding_element(&self, id: NodeId) -> bool {
        let root = self.root_declaration(id);
        self.node(root).kind() == SyntaxKind::VariableDeclaration
            && self.parent_kind(root) == Some(SyntaxKind::CatchClause)
    }

    /// Returns whether the declaration is, or is destructured from, a parameter.
    #[must_use]
    pub fn is_part_of_parameter_declaration(&self, id: NodeId) -> bool {
        self.node(self.root_declaration(id)).kind() == SyntaxKind::Parameter
    }

    /// Returns whether a parameter declares a class property through an accessibility or
    /// `readonly` modifier on a constructor.
    #[must_use]
    pub fn is_parameter_property_declaration(&self, id: NodeId, parent: NodeId) -> bool {
        self.node(id).kind() == SyntaxKind::Parameter
            && self.has_syntactic_modifier(id, ModifierFlags::PARAMETER_PROPERTY_MODIFIER)
            && self.node(parent).kind() == SyntaxKind::Constructor
    }

    /// Returns whether a module declaration is named by a string or is a `global` augmentation.
    #[must_use]
    pub fn is_ambient_module(&self, id: NodeId) -> bool {
        self.node(id)
            .data()
            .as_module_declaration()
            .is_some_and(|module| {
                self.node(module.name).kind() == SyntaxKind::StringLiteral
                    || module.keyword == SyntaxKind::GlobalKeyword
            })
    }

    /// Returns whether a module declaration is a `declare global` augmentation.
    #[must_use]
    pub fn is_global_scope_augmentation(&self, id: NodeId) -> bool {
        self.node(id)
            .data()
            .as_module_declaration()
            .is_some_and(|module| module.keyword == SyntaxKind::GlobalKeyword)
    }

    /// Returns whether the node occupies no source text, as parser-synthesized missing nodes do.
    #[must_use]
    pub fn node_is_missing(&self, id: NodeId) -> bool {
        let node = self.node(id);
        node.pos() == node.end() && node.kind() != SyntaxKind::EndOfFile
    }

    /// Returns the name node of a declaration, including the name a function or class expression
    /// is assigned to.
    #[must_use]
    pub fn name_of_declaration(&self, id: NodeId) -> Option<NodeId> {
        self.non_assigned_name_of_declaration(id).or_else(|| {
            matches!(
                self.node(id).kind(),
                SyntaxKind::FunctionExpression
                    | SyntaxKind::ArrowFunction
                    | SyntaxKind::ClassExpression
            )
            .then(|| self.assigned_name(id))
            .flatten()
        })
    }

    /// Returns the declaration's own name node.
    ///
    /// JavaScript assignment declarations are not yet bound, so binary and call expressions have
    /// no declaration name.
    #[must_use]
    pub fn non_assigned_name_of_declaration(&self, id: NodeId) -> Option<NodeId> {
        match self.node(id).data() {
            NodeData::BinaryExpression(_) | NodeData::CallExpression(_) => None,
            NodeData::ExportAssignment(assignment) => (self.node(assignment.expression).kind()
                == SyntaxKind::Identifier)
                .then_some(assignment.expression),
            data => data.name(),
        }
    }

    /// Returns the name a function or class expression is assigned to by its parent.
    #[must_use]
    pub fn assigned_name(&self, id: NodeId) -> Option<NodeId> {
        let parent = self.node(id).parent()?;
        match self.node(parent).data() {
            NodeData::PropertyAssignment(assignment) => Some(assignment.name),
            NodeData::BindingElement(element) => element.name,
            NodeData::BinaryExpression(binary) if binary.right == id => {
                match self.node(binary.left).data() {
                    NodeData::Identifier(_) => Some(binary.left),
                    NodeData::PropertyAccessExpression(access) => Some(access.name),
                    NodeData::ElementAccessExpression(access) => {
                        let argument = self.skip_parentheses(access.argument_expression);
                        self.is_string_or_numeric_literal_like(argument)
                            .then_some(argument)
                    }
                    _ => None,
                }
            }
            NodeData::VariableDeclaration(declaration)
                if self.node(declaration.name).kind() == SyntaxKind::Identifier =>
            {
                Some(declaration.name)
            }
            _ => None,
        }
    }

    /// Returns whether the declaration's name is computed from a non-literal expression.
    #[must_use]
    pub fn has_dynamic_name(&self, id: NodeId) -> bool {
        self.name_of_declaration(id)
            .is_some_and(|name| self.is_dynamic_name(name))
    }

    /// Returns whether a computed name or element access key is not a literal.
    #[must_use]
    pub fn is_dynamic_name(&self, name: NodeId) -> bool {
        let expression = match self.node(name).data() {
            NodeData::ComputedPropertyName(computed) => computed.expression,
            NodeData::ElementAccessExpression(access) => {
                self.skip_parentheses(access.argument_expression)
            }
            _ => return false,
        };
        !self.is_string_or_numeric_literal_like(expression)
            && !self.is_signed_numeric_literal(expression)
    }

    /// Returns whether the node is a string, numeric, or no-substitution template literal.
    #[must_use]
    pub fn is_string_or_numeric_literal_like(&self, id: NodeId) -> bool {
        matches!(
            self.node(id).kind(),
            SyntaxKind::StringLiteral
                | SyntaxKind::NoSubstitutionTemplateLiteral
                | SyntaxKind::NumericLiteral
        )
    }

    /// Returns whether the node is a `+` or `-` prefixed numeric literal.
    #[must_use]
    pub fn is_signed_numeric_literal(&self, id: NodeId) -> bool {
        self.node(id)
            .data()
            .as_prefix_unary_expression()
            .is_some_and(|unary| {
                matches!(
                    unary.operator,
                    SyntaxKind::PlusToken | SyntaxKind::MinusToken
                ) && self.node(unary.operand).kind() == SyntaxKind::NumericLiteral
            })
    }

    /// Returns whether the node is an identifier or a string, template, or numeric literal.
    #[must_use]
    pub fn is_property_name_literal(&self, id: NodeId) -> bool {
        self.node(id).kind() == SyntaxKind::Identifier || self.is_string_or_numeric_literal_like(id)
    }

    /// Returns the expression inside any parentheses.
    #[must_use]
    pub fn skip_parentheses(&self, mut id: NodeId) -> NodeId {
        while let Some(parenthesized) = self.node(id).data().as_parenthesized_expression() {
            id = parenthesized.expression;
        }
        id
    }

    /// Returns an identifier's text.
    #[must_use]
    pub fn identifier_text(&self, id: NodeId) -> Option<&str> {
        self.node(id)
            .data()
            .as_identifier()
            .map(|identifier| &*identifier.text)
    }

    /// Returns the text of an identifier, private identifier, or literal, as TypeScript-Go's
    /// `Node.Text` does.
    #[must_use]
    pub fn node_text(&self, id: NodeId) -> Option<&str> {
        match self.node(id).data() {
            NodeData::Identifier(data) => Some(&data.text),
            NodeData::PrivateIdentifier(data) => Some(&data.text),
            NodeData::StringLiteral(data) => Some(&data.text),
            NodeData::NumericLiteral(data) => Some(&data.text),
            NodeData::BigIntLiteral(data) => Some(&data.text),
            NodeData::NoSubstitutionTemplateLiteral(data) => Some(&data.text),
            _ => None,
        }
    }

    fn parent_kind(&self, id: NodeId) -> Option<SyntaxKind> {
        self.node(id)
            .parent()
            .map(|parent| self.node(parent).kind())
    }

    fn grandparent(&self, id: NodeId) -> NodeId {
        self.node(id)
            .parent()
            .and_then(|parent| self.node(parent).parent())
            .expect("a binding element is nested in a binding pattern and its declaration")
    }

    /// Visits the root declaration and, for variables, its declaration list and statement.
    fn for_each_combined_declaration(&self, id: NodeId, mut visit: impl FnMut(NodeId)) {
        let mut node = Some(self.root_declaration(id));
        if let Some(declaration) = node {
            visit(declaration);
            if self.node(declaration).kind() == SyntaxKind::VariableDeclaration {
                node = self.node(declaration).parent();
            }
        }
        if let Some(list) =
            node.filter(|&list| self.node(list).kind() == SyntaxKind::VariableDeclarationList)
        {
            visit(list);
            node = self.node(list).parent();
        }
        if let Some(statement) =
            node.filter(|&statement| self.node(statement).kind() == SyntaxKind::VariableStatement)
        {
            visit(statement);
        }
    }
}
