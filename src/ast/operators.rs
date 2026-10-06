//! Operator precedence and modifier classification, matching TypeScript-Go's `ast` utilities.

use super::{ModifierFlags, SyntaxKind};

/// The binding strength of an operator; higher values bind more tightly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OperatorPrecedence {
    /// Not an operator.
    Invalid,
    /// `,`
    Comma,
    /// `...`
    Spread,
    /// `yield`
    Yield,
    /// `=` and compound assignment.
    Assignment,
    /// `?:`
    Conditional,
    /// `||` and `??`.
    LogicalOr,
    /// `&&`
    LogicalAnd,
    /// `|`
    BitwiseOr,
    /// `^`
    BitwiseXor,
    /// `&`
    BitwiseAnd,
    /// `==`, `!=`, `===`, and `!==`.
    Equality,
    /// `<`, `>`, `<=`, `>=`, `instanceof`, `in`, `as`, and `satisfies`.
    Relational,
    /// `<<`, `>>`, and `>>>`.
    Shift,
    /// `+` and `-`.
    Additive,
    /// `*`, `/`, and `%`.
    Multiplicative,
    /// `**`
    Exponentiation,
    /// Prefix unary operators.
    Unary,
    /// `++` and `--`.
    Update,
    /// Calls and `new` with arguments.
    LeftHandSide,
    /// `?.`
    OptionalChain,
    /// Member access.
    Member,
    /// Primary expressions.
    Primary,
    /// Parenthesized expressions.
    Parentheses,
}

impl OperatorPrecedence {
    /// The precedence of `??`, which shares the level of `||`.
    pub const COALESCE: Self = Self::LogicalOr;
    /// The lowest real precedence.
    pub const LOWEST: Self = Self::Comma;
    /// The highest precedence.
    pub const HIGHEST: Self = Self::Parentheses;
    /// The lowest precedence at which a comma is not allowed.
    pub const DISALLOW_COMMA: Self = Self::Yield;
}

impl SyntaxKind {
    /// Returns the precedence of the kind as a binary operator.
    #[must_use]
    pub const fn binary_operator_precedence(self) -> OperatorPrecedence {
        match self {
            Self::QuestionQuestionToken => OperatorPrecedence::COALESCE,
            Self::BarBarToken => OperatorPrecedence::LogicalOr,
            Self::AmpersandAmpersandToken => OperatorPrecedence::LogicalAnd,
            Self::BarToken => OperatorPrecedence::BitwiseOr,
            Self::CaretToken => OperatorPrecedence::BitwiseXor,
            Self::AmpersandToken => OperatorPrecedence::BitwiseAnd,
            Self::EqualsEqualsToken
            | Self::ExclamationEqualsToken
            | Self::EqualsEqualsEqualsToken
            | Self::ExclamationEqualsEqualsToken => OperatorPrecedence::Equality,
            Self::LessThanToken
            | Self::GreaterThanToken
            | Self::LessThanEqualsToken
            | Self::GreaterThanEqualsToken
            | Self::InstanceOfKeyword
            | Self::InKeyword
            | Self::AsKeyword
            | Self::SatisfiesKeyword => OperatorPrecedence::Relational,
            Self::LessThanLessThanToken
            | Self::GreaterThanGreaterThanToken
            | Self::GreaterThanGreaterThanGreaterThanToken => OperatorPrecedence::Shift,
            Self::PlusToken | Self::MinusToken => OperatorPrecedence::Additive,
            Self::AsteriskToken | Self::SlashToken | Self::PercentToken => {
                OperatorPrecedence::Multiplicative
            }
            Self::AsteriskAsteriskToken => OperatorPrecedence::Exponentiation,
            _ => OperatorPrecedence::Invalid,
        }
    }

    /// Returns the modifier flag the kind represents, or none.
    #[must_use]
    pub const fn modifier_flag(self) -> ModifierFlags {
        match self {
            Self::StaticKeyword => ModifierFlags::STATIC,
            Self::PublicKeyword => ModifierFlags::PUBLIC,
            Self::ProtectedKeyword => ModifierFlags::PROTECTED,
            Self::PrivateKeyword => ModifierFlags::PRIVATE,
            Self::AbstractKeyword => ModifierFlags::ABSTRACT,
            Self::AccessorKeyword => ModifierFlags::ACCESSOR,
            Self::ExportKeyword => ModifierFlags::EXPORT,
            Self::DeclareKeyword => ModifierFlags::AMBIENT,
            Self::ConstKeyword => ModifierFlags::CONST,
            Self::DefaultKeyword => ModifierFlags::DEFAULT,
            Self::AsyncKeyword => ModifierFlags::ASYNC,
            Self::ReadonlyKeyword => ModifierFlags::READONLY,
            Self::OverrideKeyword => ModifierFlags::OVERRIDE,
            Self::InKeyword => ModifierFlags::IN,
            Self::OutKeyword => ModifierFlags::OUT,
            Self::Decorator => ModifierFlags::DECORATOR,
            _ => ModifierFlags::NONE,
        }
    }

    /// Returns whether the kind can make a constructor parameter a property.
    #[must_use]
    pub const fn is_parameter_property_modifier(self) -> bool {
        self.modifier_flag()
            .intersects(ModifierFlags::PARAMETER_PROPERTY_MODIFIER)
    }

    /// Returns whether the kind is a modifier that only applies to class members.
    #[must_use]
    pub const fn is_class_member_modifier(self) -> bool {
        self.is_parameter_property_modifier()
            || matches!(
                self,
                Self::StaticKeyword | Self::OverrideKeyword | Self::AccessorKeyword
            )
    }
}
