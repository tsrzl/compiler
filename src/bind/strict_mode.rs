//! Strict-mode and contextual-identifier grammar checks, modeled on TypeScript-Go's binder.
//!
//! TypeScript 7 treats all code as strict, so these checks apply to every file. They run in the
//! binder because it already visits every node.

use super::binder::Binder;
use crate::ast::{NodeData, NodeFlags, NodeId, SyntaxKind};
use crate::diagnostics::{self, Message};
use crate::scanner::identifier_token;

impl Binder<'_> {
    /// Runs the grammar check for `node`'s kind, if it has one.
    pub(super) fn check_strict_mode_node(&mut self, node: NodeId) {
        let ast = self.ast;
        match ast.node(node).data() {
            NodeData::Identifier(_) => self.check_contextual_identifier(node),
            NodeData::PrivateIdentifier(identifier) => {
                if &*identifier.text == "#constructor" && !self.has_parse_diagnostics() {
                    self.error_on_node(
                        node,
                        diagnostics::X_CONSTRUCTOR_IS_A_RESERVED_WORD,
                        &["#constructor"],
                    );
                }
            }
            NodeData::BinaryExpression(binary) => {
                let operator = ast.node(binary.operator_token).kind();
                // The left side of an assignment cannot be `eval` or `arguments`.
                if ast
                    .node(binary.left)
                    .kind()
                    .is_left_hand_side_expression_kind()
                    && operator.is_assignment_operator()
                {
                    self.check_strict_mode_eval_or_arguments(node, Some(binary.left));
                }
            }
            NodeData::CatchClause(clause) => {
                let name = clause
                    .variable_declaration
                    .and_then(|declaration| ast.node(declaration).data().name());
                self.check_strict_mode_eval_or_arguments(node, name);
            }
            NodeData::DeleteExpression(delete) => {
                if ast.node(delete.expression).kind() == SyntaxKind::Identifier {
                    self.error_on_node(
                        delete.expression,
                        diagnostics::X_DELETE_CANNOT_BE_CALLED_ON_AN_IDENTIFIER_IN_STRICT_MODE,
                        &[],
                    );
                }
            }
            NodeData::PostfixUnaryExpression(unary) => {
                self.check_strict_mode_eval_or_arguments(node, Some(unary.operand));
            }
            NodeData::PrefixUnaryExpression(unary) => {
                if matches!(
                    unary.operator,
                    SyntaxKind::PlusPlusToken | SyntaxKind::MinusMinusToken
                ) {
                    self.check_strict_mode_eval_or_arguments(node, Some(unary.operand));
                }
            }
            NodeData::WithStatement(_) => {
                self.error_on_first_token(
                    node,
                    diagnostics::X_WITH_STATEMENTS_ARE_NOT_ALLOWED_IN_STRICT_MODE,
                    &[],
                );
            }
            NodeData::LabeledStatement(statement)
                if is_declaration_statement_kind(ast.node(statement.statement).kind()) =>
            {
                self.error_on_first_token(
                    statement.label,
                    diagnostics::A_LABEL_IS_NOT_ALLOWED_HERE,
                    &[],
                );
            }
            _ => {}
        }
    }

    /// Reports a reserved word used as an identifier, and `await` or `yield` where they are
    /// keywords. Files with parse errors are skipped.
    fn check_contextual_identifier(&mut self, node: NodeId) {
        let ast = self.ast;
        let flags = ast.node(node).flags();
        if self.has_parse_diagnostics()
            || flags.intersects(NodeFlags::AMBIENT | NodeFlags::JSDOC)
            || self.is_identifier_name(node)
        {
            return;
        }
        let text = ast.identifier_text(node).unwrap_or_default();
        let keyword = identifier_token(text);
        let name = self.declaration_name_text(node);
        if (SyntaxKind::FIRST_FUTURE_RESERVED_WORD..=SyntaxKind::LAST_FUTURE_RESERVED_WORD)
            .contains(&keyword)
        {
            let message = self.strict_mode_message(
                node,
                diagnostics::IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_CLASS_DEFINITIONS_ARE_AUTOMATICALLY_IN_STRICT_MODE,
                diagnostics::IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE_MODULES_ARE_AUTOMATICALLY_IN_STRICT_MODE,
                diagnostics::IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_IN_STRICT_MODE,
            );
            self.error_on_node(node, message, &[name]);
        } else if keyword == SyntaxKind::AwaitKeyword {
            if self.is_external_module() && self.is_in_top_level_context(node) {
                self.error_on_node(
                    node,
                    diagnostics::IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_AT_THE_TOP_LEVEL_OF_A_MODULE,
                    &[name],
                );
            } else if flags.intersects(NodeFlags::AWAIT_CONTEXT) {
                self.error_on_node(
                    node,
                    diagnostics::IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE,
                    &[name],
                );
            }
        } else if keyword == SyntaxKind::YieldKeyword && flags.intersects(NodeFlags::YIELD_CONTEXT)
        {
            self.error_on_node(
                node,
                diagnostics::IDENTIFIER_EXPECTED_0_IS_A_RESERVED_WORD_THAT_CANNOT_BE_USED_HERE,
                &[name],
            );
        }
    }

    /// Reports `eval` or `arguments` used as a binding or assignment target.
    pub(super) fn check_strict_mode_eval_or_arguments(
        &mut self,
        context: NodeId,
        name: Option<NodeId>,
    ) {
        let ast = self.ast;
        let Some(name) = name else {
            return;
        };
        let Some(text) = ast
            .identifier_text(name)
            .filter(|text| matches!(*text, "eval" | "arguments"))
        else {
            return;
        };
        let message = self.strict_mode_message(
            context,
            diagnostics::CODE_CONTAINED_IN_A_CLASS_IS_EVALUATED_IN_JAVASCRIPT_S_STRICT_MODE_WHICH_DOES_NOT_ALLOW_THIS_USE_OF_0_FOR_MORE_INFORMATION_SEE_HTTPS_COLON_SLASH_SLASHDEVELOPER_MOZILLA_ORG_SLASHEN_US_SLASHDOCS_SLASHWEB_SLASHJAVASCRIPT_SLASHREFERENCE_SLASHSTRICT_MODE,
            diagnostics::INVALID_USE_OF_0_MODULES_ARE_AUTOMATICALLY_IN_STRICT_MODE,
            diagnostics::INVALID_USE_OF_0_IN_STRICT_MODE,
        );
        self.error_on_node(name, message, &[text]);
    }

    /// Checks a function's name unless it is ambient.
    pub(super) fn check_strict_mode_function_name(&mut self, node: NodeId) {
        if !self.ast.node(node).flags().intersects(NodeFlags::AMBIENT) {
            let name = self.ast.node(node).data().name();
            self.check_strict_mode_eval_or_arguments(node, name);
        }
    }

    /// Picks the message explaining why code is strict: it is in a class, in a module, or
    /// strict by default.
    fn strict_mode_message(
        &self,
        node: NodeId,
        in_class: Message,
        in_module: Message,
        otherwise: Message,
    ) -> Message {
        if self.containing_class(node).is_some() {
            in_class
        } else if self.is_external_module() {
            in_module
        } else {
            otherwise
        }
    }

    fn containing_class(&self, node: NodeId) -> Option<NodeId> {
        let ast = self.ast;
        std::iter::successors(ast.node(node).parent(), |&id| ast.node(id).parent()).find(|&id| {
            matches!(
                ast.node(id).kind(),
                SyntaxKind::ClassDeclaration | SyntaxKind::ClassExpression
            )
        })
    }

    /// Returns whether an identifier names a property, member, or export rather than binding or
    /// referencing a variable.
    fn is_identifier_name(&self, node: NodeId) -> bool {
        let ast = self.ast;
        let Some(parent) = ast.node(node).parent() else {
            return false;
        };
        let data = ast.node(parent).data();
        match ast.node(parent).kind() {
            SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::EnumMember
            | SyntaxKind::PropertyAssignment
            | SyntaxKind::PropertyAccessExpression => data.name() == Some(node),
            SyntaxKind::QualifiedName => data
                .as_qualified_name()
                .is_some_and(|name| name.right == node),
            SyntaxKind::BindingElement | SyntaxKind::ImportSpecifier => {
                data.property_name() == Some(node)
            }
            SyntaxKind::ExportSpecifier
            | SyntaxKind::JsxAttribute
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxOpeningElement
            | SyntaxKind::JsxClosingElement => true,
            _ => false,
        }
    }

    /// Returns whether `node` is evaluated directly at the top level of the file, where a class
    /// or function name binds in the surrounding scope.
    fn is_in_top_level_context(&self, mut node: NodeId) -> bool {
        let ast = self.ast;
        if let Some(parent) = ast.node(node).parent()
            && matches!(
                ast.node(parent).kind(),
                SyntaxKind::ClassDeclaration | SyntaxKind::FunctionDeclaration
            )
            && ast.node(parent).data().name() == Some(node)
        {
            node = parent;
        }
        self.this_container_including_arrows(node) == ast.root()
    }

    /// Returns the nearest enclosing node that binds `this`, treating arrow functions as
    /// containers, as TypeScript-Go's `GetThisContainer(node, true, false)` does.
    fn this_container_including_arrows(&self, mut node: NodeId) -> NodeId {
        let ast = self.ast;
        loop {
            node = ast
                .node(node)
                .parent()
                .expect("a non-root node has a parent");
            match ast.node(node).kind() {
                SyntaxKind::ComputedPropertyName => {
                    node = ast
                        .node(node)
                        .parent()
                        .expect("a computed property name belongs to a member");
                }
                SyntaxKind::Decorator => {
                    // A decorator's `this` resolves from the class: step to the decorated class
                    // element, which for a parameter decorator is the parameter's owner.
                    let parent = ast.node(node).parent().expect("a decorator has a parent");
                    let member = ast.node(parent).parent();
                    if ast.node(parent).kind() == SyntaxKind::Parameter
                        && let Some(member) =
                            member.filter(|&member| is_class_element_kind(ast.node(member).kind()))
                    {
                        node = member;
                    } else if is_class_element_kind(ast.node(parent).kind()) {
                        node = parent;
                    }
                }
                SyntaxKind::ArrowFunction
                | SyntaxKind::FunctionDeclaration
                | SyntaxKind::FunctionExpression
                | SyntaxKind::ModuleDeclaration
                | SyntaxKind::ClassStaticBlockDeclaration
                | SyntaxKind::PropertyDeclaration
                | SyntaxKind::PropertySignature
                | SyntaxKind::MethodDeclaration
                | SyntaxKind::MethodSignature
                | SyntaxKind::Constructor
                | SyntaxKind::GetAccessor
                | SyntaxKind::SetAccessor
                | SyntaxKind::CallSignature
                | SyntaxKind::ConstructSignature
                | SyntaxKind::IndexSignature
                | SyntaxKind::EnumDeclaration
                | SyntaxKind::SourceFile => return node,
                _ => {}
            }
        }
    }
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

/// Returns whether a labeled statement's body is a declaration or variable statement, which
/// cannot be labeled.
const fn is_declaration_statement_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::FunctionDeclaration
            | SyntaxKind::MissingDeclaration
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::JSTypeAliasDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ImportDeclaration
            | SyntaxKind::JSImportDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ExportDeclaration
            | SyntaxKind::ExportAssignment
            | SyntaxKind::NamespaceExportDeclaration
            | SyntaxKind::VariableStatement
    )
}
