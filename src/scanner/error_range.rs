//! Diagnostic ranges for nodes, modeled on TypeScript-Go's `GetErrorRangeForNode`.

use std::ops::Range;

use super::chars::is_line_break;
use super::{LanguageVariant, Scanner, skip_trivia};
use crate::ast::{Ast, NodeData, NodeFlags, NodeId, SyntaxKind};

/// Returns the UTF-8 byte range of the first token at or after `pos`, skipping trivia.
#[must_use]
pub fn range_of_token_at_position(
    text: &str,
    language_variant: LanguageVariant,
    pos: usize,
) -> Range<usize> {
    let scanner = scanner_at(text, language_variant, pos);
    scanner.token_start()..scanner.token_end()
}

/// Returns the UTF-8 byte range a diagnostic about `node` should cover.
///
/// Declarations report on their names, statements such as `return` on their keyword, and other
/// nodes on their full text without leading trivia.
#[must_use]
pub fn error_range_for_node(
    ast: &Ast,
    text: &str,
    language_variant: LanguageVariant,
    node: NodeId,
) -> Range<usize> {
    let current = ast.node(node);
    let reparsed = current.flags().intersects(NodeFlags::REPARSED);
    let error_node = match current.kind() {
        SyntaxKind::SourceFile => {
            let pos = skip_trivia(text, 0);
            if pos == text.len() {
                return 0..0;
            }
            return range_of_token_at_position(text, language_variant, pos);
        }
        SyntaxKind::FunctionDeclaration | SyntaxKind::MethodDeclaration if !reparsed => {
            ast.name_of_declaration(node)
        }
        SyntaxKind::VariableDeclaration
        | SyntaxKind::BindingElement
        | SyntaxKind::ClassDeclaration
        | SyntaxKind::InterfaceDeclaration
        | SyntaxKind::ModuleDeclaration
        | SyntaxKind::EnumDeclaration
        | SyntaxKind::EnumMember
        | SyntaxKind::FunctionExpression
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::TypeAliasDeclaration
        | SyntaxKind::JSTypeAliasDeclaration
        | SyntaxKind::PropertyDeclaration
        | SyntaxKind::PropertySignature
        | SyntaxKind::NamespaceImport => ast.name_of_declaration(node),
        SyntaxKind::ClassExpression => current.data().name(),
        SyntaxKind::ArrowFunction => return arrow_function_range(ast, text, node),
        SyntaxKind::CaseClause | SyntaxKind::DefaultClause => {
            let start = skip_trivia(text, current.pos() as usize);
            let end = current
                .data()
                .statements()
                .and_then(|statements| ast.list(statements).first().copied())
                .map_or(current.end(), |statement| ast.node(statement).pos());
            return start..end as usize;
        }
        SyntaxKind::ReturnStatement | SyntaxKind::YieldExpression => {
            let pos = skip_trivia(text, current.pos() as usize);
            return range_of_token_at_position(text, language_variant, pos);
        }
        SyntaxKind::SatisfiesExpression => {
            let NodeData::SatisfiesExpression(satisfies) = current.data() else {
                unreachable!("a satisfies expression node carries satisfies data");
            };
            let pos = skip_trivia(text, ast.node(satisfies.expression).end() as usize);
            return range_of_token_at_position(text, language_variant, pos);
        }
        SyntaxKind::Constructor if !reparsed => {
            return constructor_range(text, language_variant, current.pos());
        }
        _ => Some(node),
    };
    let Some(error_node) = error_node else {
        return range_of_token_at_position(text, language_variant, current.pos() as usize);
    };
    let error = ast.node(error_node);
    let mut pos = error.pos() as usize;
    if !ast.node_is_missing(error_node) && error.kind() != SyntaxKind::JsxText {
        pos = skip_trivia(text, pos);
    }
    pos..error.end() as usize
}

fn scanner_at(text: &str, language_variant: LanguageVariant, pos: usize) -> Scanner<'_> {
    let mut scanner = Scanner::new(text).with_language_variant(language_variant);
    scanner.reset_pos(pos);
    scanner.scan();
    scanner
}

/// Spans modifiers through the `constructor` keyword or its quoted-name form.
fn constructor_range(text: &str, language_variant: LanguageVariant, pos: u32) -> Range<usize> {
    let mut scanner = scanner_at(text, language_variant, pos as usize);
    let start = scanner.token_start();
    while !matches!(
        scanner.token(),
        SyntaxKind::ConstructorKeyword | SyntaxKind::StringLiteral | SyntaxKind::EndOfFile
    ) {
        scanner.scan();
    }
    start..scanner.token_end()
}

/// Spans an arrow function, through the end of its first line when its block body spans lines.
fn arrow_function_range(ast: &Ast, text: &str, node: NodeId) -> Range<usize> {
    let arrow = ast.node(node);
    let pos = skip_trivia(text, arrow.pos() as usize);
    if let Some(body) = arrow.data().body().map(|body| ast.node(body))
        && body.kind() == SyntaxKind::Block
    {
        let body_text = &text[body.pos() as usize..body.end() as usize];
        if let Some(offset) = body_text.find(is_line_break) {
            return pos..body.pos() as usize + offset + 1;
        }
    }
    pos..arrow.end() as usize
}
