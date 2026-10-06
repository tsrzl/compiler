//! Token-level predicates that decide which production to parse, including speculative scans.

use crate::ast::{NodeFlags, OperatorPrecedence, SyntaxKind};

use super::Parser;

impl Parser<'_> {
    pub(super) fn can_parse_semicolon(&self) -> bool {
        matches!(
            self.token,
            SyntaxKind::SemicolonToken | SyntaxKind::CloseBraceToken | SyntaxKind::EndOfFile
        ) || self.has_preceding_line_break()
    }

    pub(super) fn is_literal_property_name(&self) -> bool {
        self.token.is_identifier_or_keyword()
            || matches!(
                self.token,
                SyntaxKind::StringLiteral | SyntaxKind::NumericLiteral | SyntaxKind::BigIntLiteral
            )
    }

    /// Returns whether the token is an identifier, treating `yield` and `await` as keywords in
    /// their contexts. Strict-mode reserved words are reported by the checker instead.
    pub(super) fn is_identifier(&self) -> bool {
        if self.token == SyntaxKind::Identifier {
            return true;
        }
        if (self.token == SyntaxKind::YieldKeyword && self.in_context(NodeFlags::YIELD_CONTEXT))
            || (self.token == SyntaxKind::AwaitKeyword && self.in_context(NodeFlags::AWAIT_CONTEXT))
        {
            return false;
        }
        self.token > SyntaxKind::LAST_RESERVED_WORD
    }

    /// Returns whether the token can name a binding; `let await` and `let yield` are allowed
    /// here and rejected by the binder.
    pub(super) fn is_binding_identifier(&self) -> bool {
        self.token == SyntaxKind::Identifier || self.token > SyntaxKind::LAST_RESERVED_WORD
    }

    pub(super) fn is_binding_identifier_or_private_identifier_or_pattern(&self) -> bool {
        matches!(
            self.token,
            SyntaxKind::OpenBraceToken
                | SyntaxKind::OpenBracketToken
                | SyntaxKind::PrivateIdentifier
        ) || self.is_binding_identifier()
    }

    pub(super) fn is_import_attribute_name(&self) -> bool {
        self.token.is_identifier_or_keyword() || self.token == SyntaxKind::StringLiteral
    }

    pub(super) fn is_binary_operator(&self) -> bool {
        if self.in_context(NodeFlags::DISALLOW_IN_CONTEXT) && self.token == SyntaxKind::InKeyword {
            return false;
        }
        self.token.binary_operator_precedence() != OperatorPrecedence::Invalid
    }

    pub(super) fn is_heritage_clause(&self) -> bool {
        matches!(
            self.token,
            SyntaxKind::ExtendsKeyword | SyntaxKind::ImplementsKeyword
        )
    }

    pub(super) fn is_heritage_clause_extends_or_implements_keyword(&mut self) -> bool {
        self.is_heritage_clause()
            && self.look_ahead(|parser| {
                parser.next_token();
                parser.is_start_of_expression()
            })
    }

    pub(super) fn is_valid_heritage_clause_object_literal(&mut self) -> bool {
        self.look_ahead(|parser| {
            if parser.next_token() != SyntaxKind::CloseBraceToken {
                return true;
            }
            // `extends {}` extends an object literal only when followed by `{`, `,`, `extends`,
            // or `implements`; otherwise the braces are the class body.
            matches!(
                parser.next_token(),
                SyntaxKind::CommaToken
                    | SyntaxKind::OpenBraceToken
                    | SyntaxKind::ExtendsKeyword
                    | SyntaxKind::ImplementsKeyword
            )
        })
    }

    pub(super) fn is_start_of_statement(&mut self) -> bool {
        match self.token {
            SyntaxKind::AtToken
            | SyntaxKind::SemicolonToken
            | SyntaxKind::OpenBraceToken
            | SyntaxKind::VarKeyword
            | SyntaxKind::LetKeyword
            | SyntaxKind::UsingKeyword
            | SyntaxKind::FunctionKeyword
            | SyntaxKind::ClassKeyword
            | SyntaxKind::EnumKeyword
            | SyntaxKind::IfKeyword
            | SyntaxKind::DoKeyword
            | SyntaxKind::WhileKeyword
            | SyntaxKind::ForKeyword
            | SyntaxKind::ContinueKeyword
            | SyntaxKind::BreakKeyword
            | SyntaxKind::ReturnKeyword
            | SyntaxKind::WithKeyword
            | SyntaxKind::SwitchKeyword
            | SyntaxKind::ThrowKeyword
            | SyntaxKind::TryKeyword
            | SyntaxKind::DebuggerKeyword
            | SyntaxKind::CatchKeyword
            | SyntaxKind::FinallyKeyword => true,
            SyntaxKind::ImportKeyword => {
                self.is_start_of_declaration()
                    || self.is_next_token_open_paren_or_less_than_or_dot()
            }
            SyntaxKind::ConstKeyword | SyntaxKind::ExportKeyword => self.is_start_of_declaration(),
            SyntaxKind::AsyncKeyword
            | SyntaxKind::DeclareKeyword
            | SyntaxKind::InterfaceKeyword
            | SyntaxKind::ModuleKeyword
            | SyntaxKind::NamespaceKeyword
            | SyntaxKind::TypeKeyword
            | SyntaxKind::GlobalKeyword
            | SyntaxKind::DeferKeyword => true,
            SyntaxKind::AccessorKeyword
            | SyntaxKind::PublicKeyword
            | SyntaxKind::PrivateKeyword
            | SyntaxKind::ProtectedKeyword
            | SyntaxKind::StaticKeyword
            | SyntaxKind::ReadonlyKeyword => {
                self.is_start_of_declaration()
                    || !self.look_ahead(Self::next_token_is_identifier_or_keyword_on_same_line)
            }
            _ => self.is_start_of_expression(),
        }
    }

    pub(super) fn is_start_of_declaration(&mut self) -> bool {
        self.look_ahead(Self::scan_start_of_declaration)
    }

    fn scan_start_of_declaration(&mut self) -> bool {
        loop {
            match self.token {
                SyntaxKind::VarKeyword
                | SyntaxKind::LetKeyword
                | SyntaxKind::ConstKeyword
                | SyntaxKind::FunctionKeyword
                | SyntaxKind::ClassKeyword
                | SyntaxKind::EnumKeyword => return true,
                SyntaxKind::UsingKeyword => return self.is_using_declaration(),
                SyntaxKind::AwaitKeyword => return self.is_await_using_declaration(),
                SyntaxKind::InterfaceKeyword
                | SyntaxKind::TypeKeyword
                | SyntaxKind::DeferKeyword => {
                    return self.next_token_is_identifier_on_same_line();
                }
                SyntaxKind::ModuleKeyword | SyntaxKind::NamespaceKeyword => {
                    return self.next_token_is_identifier_or_string_literal_on_same_line();
                }
                SyntaxKind::AbstractKeyword
                | SyntaxKind::AccessorKeyword
                | SyntaxKind::AsyncKeyword
                | SyntaxKind::DeclareKeyword
                | SyntaxKind::PrivateKeyword
                | SyntaxKind::ProtectedKeyword
                | SyntaxKind::PublicKeyword
                | SyntaxKind::ReadonlyKeyword => {
                    let previous = self.token;
                    self.next_token();
                    if self.has_preceding_line_break() {
                        return false;
                    }
                    if previous == SyntaxKind::DeclareKeyword
                        && self.token == SyntaxKind::TypeKeyword
                    {
                        return true;
                    }
                }
                SyntaxKind::GlobalKeyword => {
                    self.next_token();
                    return matches!(
                        self.token,
                        SyntaxKind::OpenBraceToken
                            | SyntaxKind::Identifier
                            | SyntaxKind::ExportKeyword
                    );
                }
                SyntaxKind::ImportKeyword => {
                    self.next_token();
                    return matches!(
                        self.token,
                        SyntaxKind::DeferKeyword
                            | SyntaxKind::StringLiteral
                            | SyntaxKind::AsteriskToken
                            | SyntaxKind::OpenBraceToken
                    ) || self.token.is_identifier_or_keyword();
                }
                SyntaxKind::ExportKeyword => {
                    self.next_token();
                    if matches!(
                        self.token,
                        SyntaxKind::EqualsToken
                            | SyntaxKind::AsteriskToken
                            | SyntaxKind::OpenBraceToken
                            | SyntaxKind::DefaultKeyword
                            | SyntaxKind::AsKeyword
                            | SyntaxKind::AtToken
                    ) {
                        return true;
                    }
                    if self.token == SyntaxKind::TypeKeyword {
                        self.next_token();
                        return matches!(
                            self.token,
                            SyntaxKind::AsteriskToken | SyntaxKind::OpenBraceToken
                        ) || (self.is_identifier() && !self.has_preceding_line_break());
                    }
                }
                SyntaxKind::StaticKeyword => {
                    self.next_token();
                }
                _ => return false,
            }
        }
    }

    pub(super) fn is_start_of_expression(&mut self) -> bool {
        if self.is_start_of_left_hand_side_expression() {
            return true;
        }
        match self.token {
            SyntaxKind::PlusToken
            | SyntaxKind::MinusToken
            | SyntaxKind::TildeToken
            | SyntaxKind::ExclamationToken
            | SyntaxKind::DeleteKeyword
            | SyntaxKind::TypeOfKeyword
            | SyntaxKind::VoidKeyword
            | SyntaxKind::PlusPlusToken
            | SyntaxKind::MinusMinusToken
            | SyntaxKind::LessThanToken
            | SyntaxKind::AwaitKeyword
            | SyntaxKind::YieldKeyword
            | SyntaxKind::PrivateIdentifier
            | SyntaxKind::AtToken => true,
            // Treat the start of a binary operator as an expression so a missing operand is
            // reported and the rest of the expression is consumed.
            _ => self.is_binary_operator() || self.is_identifier(),
        }
    }

    pub(super) fn is_start_of_left_hand_side_expression(&mut self) -> bool {
        match self.token {
            SyntaxKind::ThisKeyword
            | SyntaxKind::SuperKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::StringLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::TemplateHead
            | SyntaxKind::OpenParenToken
            | SyntaxKind::OpenBracketToken
            | SyntaxKind::OpenBraceToken
            | SyntaxKind::FunctionKeyword
            | SyntaxKind::ClassKeyword
            | SyntaxKind::NewKeyword
            | SyntaxKind::SlashToken
            | SyntaxKind::SlashEqualsToken
            | SyntaxKind::Identifier => true,
            SyntaxKind::ImportKeyword => self.is_next_token_open_paren_or_less_than_or_dot(),
            _ => self.is_identifier(),
        }
    }

    pub(super) fn is_start_of_type(&mut self, in_start_of_parameter: bool) -> bool {
        match self.token {
            SyntaxKind::AnyKeyword
            | SyntaxKind::UnknownKeyword
            | SyntaxKind::StringKeyword
            | SyntaxKind::NumberKeyword
            | SyntaxKind::BigIntKeyword
            | SyntaxKind::BooleanKeyword
            | SyntaxKind::ReadonlyKeyword
            | SyntaxKind::SymbolKeyword
            | SyntaxKind::UniqueKeyword
            | SyntaxKind::VoidKeyword
            | SyntaxKind::UndefinedKeyword
            | SyntaxKind::NullKeyword
            | SyntaxKind::ThisKeyword
            | SyntaxKind::TypeOfKeyword
            | SyntaxKind::NeverKeyword
            | SyntaxKind::OpenBraceToken
            | SyntaxKind::OpenBracketToken
            | SyntaxKind::LessThanToken
            | SyntaxKind::BarToken
            | SyntaxKind::AmpersandToken
            | SyntaxKind::NewKeyword
            | SyntaxKind::StringLiteral
            | SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::TrueKeyword
            | SyntaxKind::FalseKeyword
            | SyntaxKind::ObjectKeyword
            | SyntaxKind::AsteriskToken
            | SyntaxKind::QuestionToken
            | SyntaxKind::ExclamationToken
            | SyntaxKind::DotDotDotToken
            | SyntaxKind::InferKeyword
            | SyntaxKind::ImportKeyword
            | SyntaxKind::AssertsKeyword
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::TemplateHead => true,
            SyntaxKind::FunctionKeyword => !in_start_of_parameter,
            SyntaxKind::MinusToken => {
                !in_start_of_parameter
                    && self.look_ahead(|parser| {
                        matches!(
                            parser.next_token(),
                            SyntaxKind::NumericLiteral | SyntaxKind::BigIntLiteral
                        )
                    })
            }
            SyntaxKind::OpenParenToken => {
                // `(` starts a type only when followed by `)`, `...`, a parameter, or a type.
                !in_start_of_parameter
                    && self.look_ahead(|parser| {
                        parser.next_token();
                        parser.token == SyntaxKind::CloseParenToken
                            || parser.is_start_of_parameter(false)
                            || parser.is_start_of_type(false)
                    })
            }
            _ => self.is_identifier(),
        }
    }

    pub(super) fn is_start_of_parameter(&mut self, is_jsdoc_parameter: bool) -> bool {
        self.token == SyntaxKind::DotDotDotToken
            || self.is_binding_identifier_or_private_identifier_or_pattern()
            || self.token.is_modifier_kind()
            || self.token == SyntaxKind::AtToken
            || self.is_start_of_type(!is_jsdoc_parameter)
    }

    pub(super) fn is_next_token_open_paren_or_less_than_or_dot(&mut self) -> bool {
        self.look_ahead(|parser| {
            matches!(
                parser.next_token(),
                SyntaxKind::OpenParenToken | SyntaxKind::LessThanToken | SyntaxKind::DotToken
            )
        })
    }

    pub(super) fn next_token_is_identifier_on_same_line(&mut self) -> bool {
        self.next_token();
        self.is_identifier() && !self.has_preceding_line_break()
    }

    pub(super) fn next_token_is_identifier_or_keyword_on_same_line(&mut self) -> bool {
        self.next_token();
        self.token.is_identifier_or_keyword() && !self.has_preceding_line_break()
    }

    fn next_token_is_identifier_or_string_literal_on_same_line(&mut self) -> bool {
        self.next_token();
        (self.is_identifier() || self.token == SyntaxKind::StringLiteral)
            && !self.has_preceding_line_break()
    }

    pub(super) fn next_token_is_token_string_literal(&mut self) -> bool {
        self.next_token() == SyntaxKind::StringLiteral
    }

    pub(super) fn is_let_declaration(&mut self) -> bool {
        self.look_ahead(|parser| {
            parser.next_token();
            parser.is_binding_identifier()
                || matches!(
                    parser.token,
                    SyntaxKind::OpenBraceToken | SyntaxKind::OpenBracketToken
                )
        })
    }

    /// `using` starts a declaration when followed by a binding identifier or `{` on the same
    /// line; array patterns are excluded because they conflict with element access.
    pub(super) fn is_using_declaration(&mut self) -> bool {
        self.look_ahead(|parser| {
            parser.next_token_is_binding_identifier_or_start_of_destructuring_on_same_line(false)
        })
    }

    pub(super) fn is_await_using_declaration(&mut self) -> bool {
        self.look_ahead(|parser| {
            parser.next_token() == SyntaxKind::UsingKeyword
                && parser
                    .next_token_is_binding_identifier_or_start_of_destructuring_on_same_line(false)
        })
    }

    pub(super) fn next_token_is_binding_identifier_or_start_of_destructuring_on_same_line(
        &mut self,
        disallow_of: bool,
    ) -> bool {
        self.next_token();
        if disallow_of && self.token == SyntaxKind::OfKeyword {
            return self.look_ahead(|parser| {
                parser.next_token();
                matches!(
                    parser.token,
                    SyntaxKind::EqualsToken | SyntaxKind::SemicolonToken | SyntaxKind::ColonToken
                )
            });
        }
        (self.is_binding_identifier() || self.token == SyntaxKind::OpenBraceToken)
            && !self.has_preceding_line_break()
    }

    /// Returns whether the tokens ahead start an interface or type-literal member.
    pub(super) fn scan_type_member_start(&mut self) -> bool {
        if matches!(
            self.token,
            SyntaxKind::OpenParenToken
                | SyntaxKind::LessThanToken
                | SyntaxKind::GetKeyword
                | SyntaxKind::SetKeyword
        ) {
            return true;
        }
        let mut id_token = false;
        // Eat all modifiers, holding on to the last in case it is the property name.
        while self.token.is_modifier_kind() {
            id_token = true;
            self.next_token();
        }
        if self.token == SyntaxKind::OpenBracketToken {
            return true;
        }
        if self.is_literal_property_name() {
            id_token = true;
            self.next_token();
        }
        id_token
            && (matches!(
                self.token,
                SyntaxKind::OpenParenToken
                    | SyntaxKind::LessThanToken
                    | SyntaxKind::QuestionToken
                    | SyntaxKind::ColonToken
                    | SyntaxKind::CommaToken
            ) || self.can_parse_semicolon())
    }

    /// Returns whether the tokens ahead start a class member.
    pub(super) fn scan_class_member_start(&mut self) -> bool {
        if self.token == SyntaxKind::AtToken {
            return true;
        }
        let mut id_token = None;
        // Eat all modifiers, holding on to the last in case it is the member name.
        while self.token.is_modifier_kind() {
            id_token = Some(self.token);
            if self.token.is_class_member_modifier() {
                return true;
            }
            self.next_token();
        }
        if self.token == SyntaxKind::AsteriskToken {
            return true;
        }
        if self.is_literal_property_name() {
            id_token = Some(self.token);
            self.next_token();
        }
        if self.token == SyntaxKind::OpenBracketToken {
            return true;
        }
        let Some(id_token) = id_token else {
            return false;
        };
        if !id_token.is_keyword_kind()
            || matches!(id_token, SyntaxKind::SetKeyword | SyntaxKind::GetKeyword)
        {
            return true;
        }
        matches!(
            self.token,
            SyntaxKind::OpenParenToken
                | SyntaxKind::LessThanToken
                | SyntaxKind::ExclamationToken
                | SyntaxKind::ColonToken
                | SyntaxKind::EqualsToken
                | SyntaxKind::QuestionToken
        ) || self.can_parse_semicolon()
    }
}
