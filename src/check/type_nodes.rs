//! Types written in type positions, modeled on TypeScript-Go's `getTypeFromTypeNode`.
//!
//! Ported so far: keywords, parenthesized types, literal types, unions, and type predicates.
//! Other type nodes resolve to the error type, TypeScript-Go's fallback, until their kinds are
//! ported. Union aliases are attached once type aliases are ported.

use super::checker::Checker;
use super::program::NodeRef;
use super::types::TypeId;
use crate::ast::{NodeData, SyntaxKind};
use crate::jsnum::from_string;

impl Checker<'_> {
    /// Returns the type a type node denotes, cached per node.
    pub fn type_from_type_node(&mut self, node: NodeRef) -> TypeId {
        if let Some(&ty) = self.type_node_links.get(&node) {
            return ty;
        }
        let ty = self.type_from_type_node_worker(node);
        self.type_node_links.insert(node, ty);
        ty
    }

    fn type_from_type_node_worker(&mut self, node: NodeRef) -> TypeId {
        let ast = self.files[node.file].parsed().ast();
        let intrinsics = *self.types.intrinsics();
        let at = |id| NodeRef {
            file: node.file,
            node: id,
        };
        match ast.node(node.node).data() {
            NodeData::ParenthesizedTypeNode(parenthesized) => {
                self.type_from_type_node(at(parenthesized.type_node))
            }
            NodeData::LiteralTypeNode(literal) => {
                if ast.node(literal.literal).kind() == SyntaxKind::NullKeyword {
                    return intrinsics.null;
                }
                let ty = self.check_literal_expression(at(literal.literal));
                self.types.regular_type_of_literal(ty)
            }
            NodeData::UnionTypeNode(union) => {
                let constituents: Vec<TypeId> = ast
                    .list(union.types)
                    .iter()
                    .map(|&constituent| self.type_from_type_node(at(constituent)))
                    .collect();
                self.types.union_type(&constituents)
            }
            NodeData::TypePredicateNode(predicate) => {
                if predicate.asserts_modifier.is_some() {
                    intrinsics.void
                } else {
                    intrinsics.boolean
                }
            }
            _ => match ast.node(node.node).kind() {
                SyntaxKind::AnyKeyword | SyntaxKind::JSDocAllType => intrinsics.any,
                SyntaxKind::UnknownKeyword => intrinsics.unknown,
                SyntaxKind::StringKeyword => intrinsics.string,
                SyntaxKind::NumberKeyword => intrinsics.number,
                SyntaxKind::BigIntKeyword => intrinsics.bigint,
                SyntaxKind::BooleanKeyword => intrinsics.boolean,
                SyntaxKind::SymbolKeyword => intrinsics.es_symbol,
                SyntaxKind::VoidKeyword => intrinsics.void,
                SyntaxKind::UndefinedKeyword => intrinsics.undefined,
                SyntaxKind::NullKeyword => intrinsics.null,
                SyntaxKind::NeverKeyword => intrinsics.never,
                SyntaxKind::ObjectKeyword => intrinsics.non_primitive,
                SyntaxKind::IntrinsicKeyword => intrinsics.intrinsic_marker,
                _ => intrinsics.error,
            },
        }
    }

    /// Returns the fresh literal type of a literal expression, as `checkExpression` does for
    /// string, number, and boolean literals and negated or positive numeric literals.
    pub(super) fn check_literal_expression(&mut self, node: NodeRef) -> TypeId {
        let ast = self.files[node.file].parsed().ast();
        let intrinsics = *self.types.intrinsics();
        match ast.node(node.node).data() {
            NodeData::StringLiteral(literal) => {
                let regular = self.types.string_literal_type(&literal.text);
                self.types.fresh_literal_type(regular)
            }
            NodeData::NoSubstitutionTemplateLiteral(literal) => {
                let regular = self.types.string_literal_type(&literal.text);
                self.types.fresh_literal_type(regular)
            }
            NodeData::NumericLiteral(literal) => {
                let regular = self.types.number_literal_type(from_string(&literal.text));
                self.types.fresh_literal_type(regular)
            }
            NodeData::PrefixUnaryExpression(unary)
                if matches!(
                    unary.operator,
                    SyntaxKind::MinusToken | SyntaxKind::PlusToken
                ) =>
            {
                let Some(operand) = ast.node(unary.operand).data().as_numeric_literal() else {
                    return intrinsics.error;
                };
                let value = from_string(&operand.text);
                let value = if unary.operator == SyntaxKind::MinusToken {
                    -value
                } else {
                    value
                };
                let regular = self.types.number_literal_type(value);
                self.types.fresh_literal_type(regular)
            }
            _ => match ast.node(node.node).kind() {
                SyntaxKind::TrueKeyword => intrinsics.true_type,
                SyntaxKind::FalseKeyword => intrinsics.false_type,
                _ => intrinsics.error,
            },
        }
    }
}
