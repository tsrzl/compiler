//! The arena syntax tree modeled on TypeScript-Go's `internal/ast` package.

mod arena;
mod flags;
mod flags_type;

pub(crate) use flags_type::flags_type;
mod kind;
mod kind_guards;
mod nodes;
mod operators;
mod visitor;

pub use arena::{Ast, AstBuilder, AstBuilderMark, ModifierList, Node, NodeId, NodeList};
pub use flags::{ModifierFlags, NodeFlags, TokenFlags};
pub use kind::SyntaxKind;
pub use nodes::*;
pub use operators::OperatorPrecedence;
pub use visitor::ChildVisitor;

impl SyntaxKind {
    /// Returns whether the kind is a keyword that cannot be used as an identifier.
    #[must_use]
    pub fn is_reserved_word(self) -> bool {
        (Self::FIRST_RESERVED_WORD..=Self::LAST_RESERVED_WORD).contains(&self)
    }

    /// Returns whether the kind is an identifier or any keyword.
    #[must_use]
    pub fn is_identifier_or_keyword(self) -> bool {
        self >= Self::Identifier
    }
}
