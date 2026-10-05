use crate::syntax::TextSpan;
use crate::syntax::ast::MemberAccessibility;

use super::Parser;

pub(super) struct ParsedMemberModifiers {
    pub(super) accessibility: Option<MemberAccessibility>,
    pub(super) readonly_span: Option<TextSpan>,
    pub(super) is_static: bool,
}

impl Parser {
    pub(super) fn parse_class_member_modifiers(&mut self) -> ParsedMemberModifiers {
        self.parse_modifiers(true)
    }

    pub(super) fn parse_parameter_property_modifiers(&mut self) -> ParsedMemberModifiers {
        self.parse_modifiers(false)
    }

    fn parse_modifiers(&mut self, allow_static: bool) -> ParsedMemberModifiers {
        let mut accessibility = None;
        let mut readonly_span = None;
        let mut is_static = false;
        loop {
            let matched_modifier = if self.matches_identifier("public") {
                accessibility = Some(MemberAccessibility::Public);
                true
            } else if self.matches_identifier("private") {
                accessibility = Some(MemberAccessibility::Private);
                true
            } else if self.matches_identifier("protected") {
                accessibility = Some(MemberAccessibility::Protected);
                true
            } else if self.matches_identifier("readonly") {
                readonly_span = self.tokens.get(self.current - 1).map(|token| token.span);
                true
            } else if allow_static && self.matches_identifier("static") {
                is_static = true;
                true
            } else {
                false
            };
            if !matched_modifier {
                break;
            }
        }
        ParsedMemberModifiers {
            accessibility,
            readonly_span,
            is_static,
        }
    }
}
