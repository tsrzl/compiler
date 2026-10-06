//! Modifiers and decorators.

use crate::ast::{Decorator, ModifierFlags, ModifierList, NodeData, NodeFlags, NodeId, SyntaxKind};
use crate::diagnostics;

use super::Parser;

/// How a modifier list is parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct ModifierOptions {
    pub(super) allow_decorators: bool,
    pub(super) permit_const_as_modifier: bool,
    pub(super) stop_on_start_of_class_static_block: bool,
}

impl ModifierOptions {
    pub(super) const DECORATED: Self = Self {
        allow_decorators: true,
        permit_const_as_modifier: false,
        stop_on_start_of_class_static_block: false,
    };
}

impl Parser<'_> {
    pub(super) fn parse_modifiers(&mut self) -> Option<ModifierList> {
        self.parse_modifiers_with(ModifierOptions::default())
    }

    /// Parses leading decorators and modifiers, then any trailing decorators and modifiers.
    pub(super) fn parse_modifiers_with(
        &mut self,
        options: ModifierOptions,
    ) -> Option<ModifierList> {
        let pos = self.node_pos();
        let mut has_leading_modifier = false;
        let mut has_trailing_decorator = false;
        let mut has_trailing_modifier = false;
        let mut has_static_modifier = false;
        let mut list = Vec::new();
        loop {
            if options.allow_decorators
                && self.token == SyntaxKind::AtToken
                && !has_trailing_modifier
            {
                list.push(self.parse_decorator());
                if has_leading_modifier {
                    has_trailing_decorator = true;
                }
                continue;
            }
            let Some(modifier) = self.try_parse_modifier(has_static_modifier, options) else {
                break;
            };
            if self.builder.node(modifier).kind() == SyntaxKind::StaticKeyword {
                has_static_modifier = true;
            }
            list.push(modifier);
            if has_trailing_decorator {
                has_trailing_modifier = true;
            } else {
                has_leading_modifier = true;
            }
        }
        if list.is_empty() {
            return None;
        }
        let end = self.node_pos();
        Some(self.new_modifier_list(pos, end, list))
    }

    pub(super) fn new_modifier_list(
        &mut self,
        pos: usize,
        end: usize,
        modifiers: Vec<NodeId>,
    ) -> ModifierList {
        let flags = modifiers
            .iter()
            .fold(ModifierFlags::NONE, |flags, &modifier| {
                flags | self.builder.node(modifier).kind().modifier_flag()
            });
        ModifierList::new(self.new_node_list(pos, end, modifiers), flags)
    }

    /// Adds `Ambient` to every modifier, as TypeScript-Go does for `declare` declarations.
    pub(super) fn mark_modifiers_ambient(&mut self, modifiers: &ModifierList) {
        let nodes = self.builder.list(*modifiers.list()).to_vec();
        for node in nodes {
            self.builder.add_flags(node, NodeFlags::AMBIENT);
        }
    }

    fn parse_decorator(&mut self) -> NodeId {
        let pos = self.node_pos();
        self.parse_expected(SyntaxKind::AtToken);
        let expression = self.do_in_context(
            NodeFlags::DECORATOR_CONTEXT,
            true,
            Self::parse_decorator_expression,
        );
        self.finish_node(
            SyntaxKind::Decorator,
            pos,
            NodeData::Decorator(Decorator { expression }),
        )
    }

    fn parse_decorator_expression(&mut self) -> NodeId {
        if self.in_context(NodeFlags::AWAIT_CONTEXT) && self.token == SyntaxKind::AwaitKeyword {
            // `@await` is disallowed in an [Await] context; parse a missing identifier and move on.
            let pos = self.node_pos();
            let await_expression =
                self.parse_identifier_with_diagnostic(Some(diagnostics::EXPRESSION_EXPECTED), None);
            self.next_token();
            let member = self.parse_member_expression_rest(pos, await_expression, true);
            return self.parse_call_expression_rest(pos, member);
        }
        self.parse_left_hand_side_expression_or_higher()
    }

    fn try_parse_modifier(
        &mut self,
        has_seen_static_modifier: bool,
        options: ModifierOptions,
    ) -> Option<NodeId> {
        let pos = self.node_pos();
        let kind = self.token;
        let is_modifier = if self.token == SyntaxKind::ConstKeyword
            && options.permit_const_as_modifier
        {
            // Later modifiers must be on the same line so that a standalone `const` declaration
            // is not misread.
            let follows = self.look_ahead(Self::next_token_is_on_same_line_and_can_follow_modifier);
            if follows {
                self.next_token();
            }
            follows
        } else if self.token == SyntaxKind::StaticKeyword
            && (has_seen_static_modifier
                || (options.stop_on_start_of_class_static_block
                    && self.look_ahead(|parser| parser.next_token() == SyntaxKind::OpenBraceToken)))
        {
            false
        } else {
            self.parse_any_contextual_modifier()
        };
        if !is_modifier {
            return None;
        }
        Some(self.finish_node(kind, pos, NodeData::Token))
    }

    /// Consumes `kind` when it acts as a modifier rather than an identifier.
    pub(super) fn parse_contextual_modifier(&mut self, kind: SyntaxKind) -> bool {
        let state = self.mark();
        if self.token == kind && self.next_token_can_follow_modifier() {
            return true;
        }
        self.rewind(state);
        false
    }

    fn parse_any_contextual_modifier(&mut self) -> bool {
        let state = self.mark();
        if self.token.is_modifier_kind() && self.next_token_can_follow_modifier() {
            return true;
        }
        self.rewind(state);
        false
    }

    fn next_token_can_follow_modifier(&mut self) -> bool {
        match self.token {
            // `const` is a modifier only before `enum`.
            SyntaxKind::ConstKeyword => self.next_token() == SyntaxKind::EnumKeyword,
            SyntaxKind::ExportKeyword => {
                self.next_token();
                if self.token == SyntaxKind::DefaultKeyword {
                    return self.look_ahead(Self::next_token_can_follow_default_keyword);
                }
                if self.token == SyntaxKind::TypeKeyword {
                    return self.look_ahead(|parser| {
                        parser.next_token();
                        parser.can_follow_export_modifier()
                    });
                }
                self.can_follow_export_modifier()
            }
            SyntaxKind::DefaultKeyword => self.next_token_can_follow_default_keyword(),
            SyntaxKind::StaticKeyword => {
                self.next_token();
                self.can_follow_modifier()
            }
            SyntaxKind::GetKeyword | SyntaxKind::SetKeyword => {
                self.next_token();
                self.token == SyntaxKind::OpenBracketToken || self.is_literal_property_name()
            }
            _ => self.next_token_is_on_same_line_and_can_follow_modifier(),
        }
    }

    fn next_token_can_follow_default_keyword(&mut self) -> bool {
        match self.next_token() {
            SyntaxKind::ClassKeyword
            | SyntaxKind::FunctionKeyword
            | SyntaxKind::InterfaceKeyword
            | SyntaxKind::AtToken => true,
            SyntaxKind::AbstractKeyword => self.look_ahead(|parser| {
                parser.next_token() == SyntaxKind::ClassKeyword
                    && !parser.has_preceding_line_break()
            }),
            SyntaxKind::AsyncKeyword => {
                self.look_ahead(Self::next_token_is_function_keyword_on_same_line)
            }
            _ => false,
        }
    }

    fn can_follow_export_modifier(&self) -> bool {
        self.token == SyntaxKind::AtToken
            || (!matches!(
                self.token,
                SyntaxKind::AsteriskToken | SyntaxKind::AsKeyword | SyntaxKind::OpenBraceToken
            ) && self.can_follow_modifier())
    }

    fn can_follow_modifier(&self) -> bool {
        matches!(
            self.token,
            SyntaxKind::OpenBracketToken
                | SyntaxKind::OpenBraceToken
                | SyntaxKind::AsteriskToken
                | SyntaxKind::DotDotDotToken
        ) || self.is_literal_property_name()
    }

    fn next_token_is_on_same_line_and_can_follow_modifier(&mut self) -> bool {
        self.next_token();
        !self.has_preceding_line_break() && self.can_follow_modifier()
    }
}
