//! The type model, modeled on TypeScript-Go's `internal/checker/types.go`.

use super::{ObjectFlags, TypeFlags};

/// A type's identity within its [`super::TypeTable`]. Identities start at 1 and follow creation
/// order, as TypeScript-Go's do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(u32);

impl TypeId {
    /// Returns the numeric identity.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }

    pub(super) const fn index(self) -> usize {
        (self.0 - 1) as usize
    }

    pub(super) fn from_index(index: usize) -> Self {
        Self(u32::try_from(index + 1).expect("type count fits in u32"))
    }
}

/// The value of a literal type.
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    /// A string literal.
    String(Box<str>),
    /// A number literal.
    Number(f64),
    /// A boolean literal.
    Boolean(bool),
}

/// The kind-specific data of a type.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeData {
    /// A built-in type such as `string`, `any`, or `never`.
    Intrinsic {
        /// The keyword or diagnostic name of the type.
        name: &'static str,
    },
    /// A literal type and its fresh and regular variants.
    Literal {
        /// The literal value.
        value: LiteralValue,
        /// The fresh variant, created on demand for literals written in expressions.
        fresh_type: Option<TypeId>,
        /// The regular variant, which is the type itself unless it is fresh.
        regular_type: TypeId,
    },
    /// A union of sorted, deduplicated constituents.
    Union {
        /// The constituent types in [`super::compare_types`] order.
        types: Vec<TypeId>,
        /// The denormalized union the type was written as, when it combines named unions.
        origin: Option<TypeId>,
    },
}

/// A type.
#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub(super) flags: TypeFlags,
    pub(super) object_flags: ObjectFlags,
    pub(super) data: TypeData,
}

impl Type {
    /// Returns the type's flags.
    #[must_use]
    pub const fn flags(&self) -> TypeFlags {
        self.flags
    }

    /// Returns the type's object flags.
    #[must_use]
    pub const fn object_flags(&self) -> ObjectFlags {
        self.object_flags
    }

    /// Returns the kind-specific data.
    #[must_use]
    pub const fn data(&self) -> &TypeData {
        &self.data
    }
}
