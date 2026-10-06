//! Punctuator scanning.

use crate::ast::SyntaxKind;

use super::{LanguageVariant, Scanner};

impl Scanner<'_> {
    pub(super) fn scan_punctuation(&mut self, byte: u8) -> Option<SyntaxKind> {
        let next = self.byte_at(1);
        let after_next = self.byte_at(2);
        let (token, length) = match (byte, next, after_next) {
            (b'!', Some(b'='), Some(b'=')) => (SyntaxKind::ExclamationEqualsEqualsToken, 3),
            (b'!', Some(b'='), _) => (SyntaxKind::ExclamationEqualsToken, 2),
            (b'!', _, _) => (SyntaxKind::ExclamationToken, 1),
            (b'%', Some(b'='), _) => (SyntaxKind::PercentEqualsToken, 2),
            (b'%', _, _) => (SyntaxKind::PercentToken, 1),
            (b'&', Some(b'&'), Some(b'=')) => (SyntaxKind::AmpersandAmpersandEqualsToken, 3),
            (b'&', Some(b'&'), _) => (SyntaxKind::AmpersandAmpersandToken, 2),
            (b'&', Some(b'='), _) => (SyntaxKind::AmpersandEqualsToken, 2),
            (b'&', _, _) => (SyntaxKind::AmpersandToken, 1),
            (b'(', _, _) => (SyntaxKind::OpenParenToken, 1),
            (b')', _, _) => (SyntaxKind::CloseParenToken, 1),
            (b'*', Some(b'='), _) => (SyntaxKind::AsteriskEqualsToken, 2),
            (b'*', Some(b'*'), Some(b'=')) => (SyntaxKind::AsteriskAsteriskEqualsToken, 3),
            (b'*', Some(b'*'), _) => (SyntaxKind::AsteriskAsteriskToken, 2),
            (b'*', _, _) => (SyntaxKind::AsteriskToken, 1),
            (b'+', Some(b'='), _) => (SyntaxKind::PlusEqualsToken, 2),
            (b'+', Some(b'+'), _) => (SyntaxKind::PlusPlusToken, 2),
            (b'+', _, _) => (SyntaxKind::PlusToken, 1),
            (b',', _, _) => (SyntaxKind::CommaToken, 1),
            (b'-', Some(b'='), _) => (SyntaxKind::MinusEqualsToken, 2),
            (b'-', Some(b'-'), _) => (SyntaxKind::MinusMinusToken, 2),
            (b'-', _, _) => (SyntaxKind::MinusToken, 1),
            (b'.', Some(b'.'), Some(b'.')) => (SyntaxKind::DotDotDotToken, 3),
            (b'.', _, _) => (SyntaxKind::DotToken, 1),
            (b'/', Some(b'='), _) => (SyntaxKind::SlashEqualsToken, 2),
            (b'/', _, _) => (SyntaxKind::SlashToken, 1),
            (b':', _, _) => (SyntaxKind::ColonToken, 1),
            (b';', _, _) => (SyntaxKind::SemicolonToken, 1),
            (b'<', Some(b'<'), Some(b'=')) => (SyntaxKind::LessThanLessThanEqualsToken, 3),
            (b'<', Some(b'<'), _) => (SyntaxKind::LessThanLessThanToken, 2),
            (b'<', Some(b'='), _) => (SyntaxKind::LessThanEqualsToken, 2),
            (b'<', Some(b'/'), next)
                if self.language_variant == LanguageVariant::Jsx && next != Some(b'*') =>
            {
                (SyntaxKind::LessThanSlashToken, 2)
            }
            (b'<', _, _) => (SyntaxKind::LessThanToken, 1),
            (b'=', Some(b'='), Some(b'=')) => (SyntaxKind::EqualsEqualsEqualsToken, 3),
            (b'=', Some(b'='), _) => (SyntaxKind::EqualsEqualsToken, 2),
            (b'=', Some(b'>'), _) => (SyntaxKind::EqualsGreaterThanToken, 2),
            (b'=', _, _) => (SyntaxKind::EqualsToken, 1),
            (b'>', _, _) => (SyntaxKind::GreaterThanToken, 1),
            (b'?', Some(b'.'), digit) if !digit.is_some_and(|digit| digit.is_ascii_digit()) => {
                (SyntaxKind::QuestionDotToken, 2)
            }
            (b'?', Some(b'?'), Some(b'=')) => (SyntaxKind::QuestionQuestionEqualsToken, 3),
            (b'?', Some(b'?'), _) => (SyntaxKind::QuestionQuestionToken, 2),
            (b'?', _, _) => (SyntaxKind::QuestionToken, 1),
            (b'[', _, _) => (SyntaxKind::OpenBracketToken, 1),
            (b']', _, _) => (SyntaxKind::CloseBracketToken, 1),
            (b'^', Some(b'='), _) => (SyntaxKind::CaretEqualsToken, 2),
            (b'^', _, _) => (SyntaxKind::CaretToken, 1),
            (b'{', _, _) => (SyntaxKind::OpenBraceToken, 1),
            (b'|', Some(b'|'), Some(b'=')) => (SyntaxKind::BarBarEqualsToken, 3),
            (b'|', Some(b'|'), _) => (SyntaxKind::BarBarToken, 2),
            (b'|', Some(b'='), _) => (SyntaxKind::BarEqualsToken, 2),
            (b'|', _, _) => (SyntaxKind::BarToken, 1),
            (b'}', _, _) => (SyntaxKind::CloseBraceToken, 1),
            (b'~', _, _) => (SyntaxKind::TildeToken, 1),
            (b'@', _, _) => (SyntaxKind::AtToken, 1),
            _ => return None,
        };
        self.state.advance(length);
        Some(token)
    }
}
