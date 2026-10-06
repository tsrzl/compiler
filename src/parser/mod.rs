//! The TypeScript parser, ported from TypeScript-Go's `internal/parser` package.
//!
//! [`parse_source_file`] turns source text into an immutable [`ParsedSourceFile`]. Positions are
//! UTF-8 byte offsets, matching TypeScript-Go.

mod arrow_functions;
mod bindings;
mod class_members;
mod classes;
mod control_flow;
mod declarations;
mod expressions;
mod identifiers;
mod lists;
mod lookahead;
mod modifiers;
mod modules;
mod signatures;
mod statements;
mod type_declarations;
mod type_members;
mod types;

use std::collections::HashSet;

use crate::ast::{
    Ast, AstBuilder, AstBuilderMark, NodeData, NodeFlags, NodeId, NodeList, SourceFile, SyntaxKind,
    TokenFlags,
};
use crate::diagnostics::{self, Message};
use crate::scanner::{CommentDirective, LanguageVariant, Scanner, ScannerState};
use crate::tspath::is_declaration_file_name;

pub use lists::ParsingContext;

/// The kind of script a file contains, matching TypeScript-Go's `core.ScriptKind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptKind {
    /// JavaScript.
    Js,
    /// JavaScript with JSX.
    Jsx,
    /// TypeScript.
    Ts,
    /// TypeScript with JSX.
    Tsx,
    /// A file with an extension supplied by a host.
    External,
    /// JSON.
    Json,
    /// A file whose kind is decided later by a host.
    Deferred,
}

impl ScriptKind {
    const fn language_variant(self) -> LanguageVariant {
        match self {
            Self::Tsx | Self::Jsx | Self::Js | Self::Json => LanguageVariant::Jsx,
            Self::Ts | Self::External | Self::Deferred => LanguageVariant::Standard,
        }
    }

    const fn is_java_script(self) -> bool {
        matches!(self, Self::Js | Self::Jsx)
    }
}

/// Inputs that control how one file is parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOptions {
    file_name: String,
    script_kind: ScriptKind,
}

impl ParseOptions {
    /// Creates options for parsing `file_name` as `script_kind`.
    #[must_use]
    pub fn new(file_name: impl Into<String>, script_kind: ScriptKind) -> Self {
        Self {
            file_name: file_name.into(),
            script_kind,
        }
    }

    /// Returns the name of the file being parsed.
    #[must_use]
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// Returns the script kind of the file being parsed.
    #[must_use]
    pub const fn script_kind(&self) -> ScriptKind {
        self.script_kind
    }
}

/// A diagnostic reported while parsing, located by UTF-8 byte offsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDiagnostic {
    message: Message,
    pos: usize,
    end: usize,
    arguments: Vec<String>,
    related: Vec<ParseDiagnostic>,
}

impl ParseDiagnostic {
    fn new(message: Message, pos: usize, end: usize, arguments: &[&str]) -> Self {
        Self {
            message,
            pos,
            end,
            arguments: arguments
                .iter()
                .map(|&argument| argument.to_owned())
                .collect(),
            related: Vec::new(),
        }
    }

    /// Returns the diagnostic message template.
    #[must_use]
    pub const fn message(&self) -> Message {
        self.message
    }

    /// Returns the UTF-8 byte offset where the diagnostic starts.
    #[must_use]
    pub const fn pos(&self) -> usize {
        self.pos
    }

    /// Returns the UTF-8 byte offset where the diagnostic ends.
    #[must_use]
    pub const fn end(&self) -> usize {
        self.end
    }

    /// Returns the formatted message text.
    #[must_use]
    pub fn text(&self) -> String {
        let arguments = self
            .arguments
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        self.message.format(&arguments)
    }

    /// Returns related locations that explain this diagnostic.
    #[must_use]
    pub fn related(&self) -> &[ParseDiagnostic] {
        &self.related
    }
}

/// The immutable result of parsing one file.
#[derive(Debug, Clone)]
pub struct ParsedSourceFile {
    options: ParseOptions,
    ast: Ast,
    diagnostics: Vec<ParseDiagnostic>,
    comment_directives: Vec<CommentDirective>,
    language_variant: LanguageVariant,
    is_declaration_file: bool,
}

impl ParsedSourceFile {
    /// Returns the options the file was parsed with.
    #[must_use]
    pub const fn options(&self) -> &ParseOptions {
        &self.options
    }

    /// Returns the syntax tree, rooted at the `SourceFile` node.
    #[must_use]
    pub const fn ast(&self) -> &Ast {
        &self.ast
    }

    /// Returns the syntax diagnostics in report order.
    #[must_use]
    pub fn diagnostics(&self) -> &[ParseDiagnostic] {
        &self.diagnostics
    }

    /// Returns the `@ts-expect-error` and `@ts-ignore` comments.
    #[must_use]
    pub fn comment_directives(&self) -> &[CommentDirective] {
        &self.comment_directives
    }

    /// Returns whether JSX syntax was recognized.
    #[must_use]
    pub const fn language_variant(&self) -> LanguageVariant {
        self.language_variant
    }

    /// Returns whether the file is a declaration file.
    #[must_use]
    pub const fn is_declaration_file(&self) -> bool {
        self.is_declaration_file
    }
}

/// Parses `text` as a source file.
#[must_use]
pub fn parse_source_file(options: &ParseOptions, text: &str) -> ParsedSourceFile {
    let mut parser = Parser::new(options, text);
    parser.next_token();
    parser.parse_source_file_worker(options)
}

/// Facts about the `JSDoc` comment preceding the current token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct JsdocScannerInfo {
    has_jsdoc: bool,
    has_deprecated: bool,
    has_see_or_link: bool,
}

/// The restorable state of a [`Parser`] for speculative parsing.
#[derive(Debug, Clone)]
struct ParserState {
    scanner: ScannerState,
    context_flags: NodeFlags,
    diagnostic_count: usize,
    builder: AstBuilderMark,
    statement_has_await_identifier: bool,
    has_parse_error: bool,
}

struct Parser<'text> {
    scanner: Scanner<'text>,
    builder: AstBuilder,
    script_kind: ScriptKind,
    language_variant: LanguageVariant,
    diagnostics: Vec<ParseDiagnostic>,
    token: SyntaxKind,
    context_flags: NodeFlags,
    source_flags: NodeFlags,
    parsing_contexts: u32,
    statement_has_await_identifier: bool,
    has_parse_error: bool,
    /// Token positions already found not to start a parenthesized arrow function.
    not_parenthesized_arrow: HashSet<usize>,
}

impl<'text> Parser<'text> {
    fn new(options: &ParseOptions, text: &'text str) -> Self {
        let script_kind = options.script_kind;
        let language_variant = script_kind.language_variant();
        let context_flags = match script_kind {
            ScriptKind::Js | ScriptKind::Jsx => NodeFlags::JAVA_SCRIPT_FILE,
            ScriptKind::Json => NodeFlags::JAVA_SCRIPT_FILE | NodeFlags::JSON_FILE,
            _ => NodeFlags::NONE,
        };
        Self {
            scanner: Scanner::new(text).with_language_variant(language_variant),
            builder: AstBuilder::new(),
            script_kind,
            language_variant,
            diagnostics: Vec::new(),
            token: SyntaxKind::Unknown,
            context_flags,
            source_flags: NodeFlags::NONE,
            parsing_contexts: 0,
            statement_has_await_identifier: false,
            has_parse_error: false,
            not_parenthesized_arrow: HashSet::new(),
        }
    }

    fn parse_source_file_worker(mut self, options: &ParseOptions) -> ParsedSourceFile {
        let is_declaration_file = is_declaration_file_name(&options.file_name);
        if is_declaration_file {
            self.context_flags |= NodeFlags::AMBIENT;
        }
        let pos = self.node_pos();
        let statements = self.parse_list_with_index(ParsingContext::SourceElements, |parser, _| {
            parser.parse_toplevel_statement()
        });
        let end = self.node_pos();
        let end_jsdoc = self.jsdoc_scanner_info();
        let end_of_file_token = self.parse_token_node();
        self.with_jsdoc(end_of_file_token, end_jsdoc);
        assert_eq!(
            self.builder.node(end_of_file_token).kind(),
            SyntaxKind::EndOfFile,
            "the scanner must end with an end-of-file token"
        );
        let statements = self.builder.add_list(to_u32(pos), to_u32(end), statements);
        let data = NodeData::SourceFile(SourceFile {
            statements,
            end_of_file_token,
        });
        let root = self.finish_node(SyntaxKind::SourceFile, pos, data);
        self.builder.add_flags(root, self.source_flags);
        let comment_directives = self.scanner.comment_directives().to_vec();
        ParsedSourceFile {
            options: options.clone(),
            ast: self.builder.finish(root),
            diagnostics: self.diagnostics,
            comment_directives,
            language_variant: self.language_variant,
            is_declaration_file,
        }
    }

    fn parse_toplevel_statement(&mut self) -> NodeId {
        self.statement_has_await_identifier = false;
        self.parse_statement()
    }

    const fn is_java_script(&self) -> bool {
        self.script_kind.is_java_script()
    }

    // Diagnostics

    fn parse_error_at(&mut self, pos: usize, end: usize, message: Message, arguments: &[&str]) {
        // Don't report another error at the same location as the previous one.
        if self.diagnostics.last().is_none_or(|last| last.pos != pos) {
            self.diagnostics
                .push(ParseDiagnostic::new(message, pos, end, arguments));
        }
        self.has_parse_error = true;
    }

    fn parse_error_at_current_token(&mut self, message: Message, arguments: &[&str]) {
        let (pos, end) = (self.scanner.token_start(), self.scanner.token_end());
        self.parse_error_at(pos, end, message, arguments);
    }

    fn drain_scanner_diagnostics(&mut self) {
        for diagnostic in self.scanner.take_diagnostics() {
            let pos = diagnostic.start();
            let end = pos + diagnostic.length();
            if self.diagnostics.last().is_none_or(|last| last.pos != pos) {
                self.diagnostics.push(ParseDiagnostic {
                    message: diagnostic.message(),
                    pos,
                    end,
                    arguments: diagnostic.arguments().to_vec(),
                    related: Vec::new(),
                });
            }
            self.has_parse_error = true;
        }
    }

    // Speculation

    fn mark(&self) -> ParserState {
        ParserState {
            scanner: self.scanner.mark(),
            context_flags: self.context_flags,
            diagnostic_count: self.diagnostics.len(),
            builder: self.builder.mark(),
            statement_has_await_identifier: self.statement_has_await_identifier,
            has_parse_error: self.has_parse_error,
        }
    }

    fn rewind(&mut self, state: ParserState) {
        self.scanner.rewind(state.scanner);
        self.token = self.scanner.token();
        self.context_flags = state.context_flags;
        self.diagnostics.truncate(state.diagnostic_count);
        self.builder.rewind(state.builder);
        self.statement_has_await_identifier = state.statement_has_await_identifier;
        self.has_parse_error = state.has_parse_error;
    }

    /// Runs `callback` speculatively and restores all parser state afterwards.
    fn look_ahead(&mut self, callback: impl FnOnce(&mut Self) -> bool) -> bool {
        let state = self.mark();
        let result = callback(self);
        self.rewind(state);
        result
    }

    // Tokens

    fn next_token(&mut self) -> SyntaxKind {
        // A keyword spelled with an escape is reported where it is consumed.
        if self.token.is_keyword_kind()
            && self
                .scanner
                .token_flags()
                .intersects(TokenFlags::UNICODE_ESCAPE | TokenFlags::EXTENDED_UNICODE_ESCAPE)
        {
            self.parse_error_at_current_token(
                diagnostics::KEYWORDS_CANNOT_CONTAIN_ESCAPE_CHARACTERS,
                &[],
            );
        }
        self.next_token_without_check()
    }

    fn next_token_without_check(&mut self) -> SyntaxKind {
        self.token = self.scanner.scan();
        self.drain_scanner_diagnostics();
        self.token
    }

    fn node_pos(&self) -> usize {
        self.scanner.token_full_start()
    }

    fn has_preceding_line_break(&self) -> bool {
        self.scanner.has_preceding_line_break()
    }

    fn jsdoc_scanner_info(&self) -> JsdocScannerInfo {
        let flags = self.scanner.token_flags();
        if !flags.intersects(TokenFlags::PRECEDING_JSDOC_COMMENT) {
            return JsdocScannerInfo::default();
        }
        JsdocScannerInfo {
            has_jsdoc: true,
            has_deprecated: flags.intersects(TokenFlags::PRECEDING_JSDOC_WITH_DEPRECATED),
            has_see_or_link: flags.intersects(TokenFlags::PRECEDING_JSDOC_WITH_SEE_OR_LINK),
        }
    }

    fn parse_optional(&mut self, token: SyntaxKind) -> bool {
        if self.token == token {
            self.next_token();
            return true;
        }
        false
    }

    fn parse_expected(&mut self, kind: SyntaxKind) -> bool {
        self.parse_expected_with_diagnostic(kind, None, true)
    }

    fn parse_expected_with_diagnostic(
        &mut self,
        kind: SyntaxKind,
        message: Option<Message>,
        should_advance: bool,
    ) -> bool {
        if self.token == kind {
            if should_advance {
                self.next_token();
            }
            return true;
        }
        match message {
            Some(message) => self.parse_error_at_current_token(message, &[]),
            None => self
                .parse_error_at_current_token(diagnostics::X_0_EXPECTED, &[token_to_string(kind)]),
        }
        false
    }

    fn parse_optional_token(&mut self, kind: SyntaxKind) -> Option<NodeId> {
        (self.token == kind).then(|| self.parse_token_node())
    }

    fn parse_expected_token(&mut self, kind: SyntaxKind) -> NodeId {
        if let Some(token) = self.parse_optional_token(kind) {
            return token;
        }
        self.parse_error_at_current_token(diagnostics::X_0_EXPECTED, &[token_to_string(kind)]);
        let pos = self.node_pos();
        self.finish_node(kind, pos, NodeData::Token)
    }

    fn try_parse_semicolon(&mut self) -> bool {
        if !self.can_parse_semicolon() {
            return false;
        }
        if self.token == SyntaxKind::SemicolonToken {
            self.next_token();
        }
        true
    }

    fn parse_semicolon(&mut self) -> bool {
        self.try_parse_semicolon() || self.parse_expected(SyntaxKind::SemicolonToken)
    }

    // Rescanning, keeping the current token and diagnostics in sync with the scanner.

    fn rescan_greater_than_token(&mut self) -> SyntaxKind {
        self.token = self.scanner.rescan_greater_than_token();
        self.token
    }

    fn rescan_less_than_token(&mut self) -> SyntaxKind {
        self.token = self.scanner.rescan_less_than_token();
        self.token
    }

    fn rescan_slash_token(&mut self) -> SyntaxKind {
        self.token = self.scanner.rescan_slash_token();
        self.drain_scanner_diagnostics();
        self.token
    }

    fn rescan_template_token(&mut self, is_tagged_template: bool) -> SyntaxKind {
        self.token = self.scanner.rescan_template_token(is_tagged_template);
        self.drain_scanner_diagnostics();
        self.token
    }

    fn parse_token_node(&mut self) -> NodeId {
        let pos = self.node_pos();
        let kind = self.token;
        self.next_token();
        self.finish_node(kind, pos, NodeData::Token)
    }

    // Nodes

    /// Adds a node spanning `pos` to the current token's full start.
    fn finish_node(&mut self, kind: SyntaxKind, pos: usize, data: NodeData) -> NodeId {
        let end = self.node_pos();
        self.finish_node_with_end(kind, pos, end, data)
    }

    /// Adds a node whose own flags, such as `OptionalChain`, combine with the context flags.
    fn finish_node_with_flags(
        &mut self,
        kind: SyntaxKind,
        pos: usize,
        flags: NodeFlags,
        data: NodeData,
    ) -> NodeId {
        let node = self.finish_node(kind, pos, data);
        self.builder.add_flags(node, flags);
        node
    }

    /// Returns whether a node has source text; missing nodes are empty and not end of file.
    fn node_is_present(&self, node: NodeId) -> bool {
        let node = self.builder.node(node);
        !(node.pos() == node.end() && node.kind() != SyntaxKind::EndOfFile)
    }

    fn skip_trivia(&self, pos: usize) -> usize {
        crate::scanner::skip_trivia(self.scanner.text(), pos)
    }

    fn finish_node_with_end(
        &mut self,
        kind: SyntaxKind,
        pos: usize,
        end: usize,
        data: NodeData,
    ) -> NodeId {
        let mut flags = self.context_flags;
        if self.has_parse_error {
            flags |= NodeFlags::THIS_NODE_HAS_ERROR;
            self.has_parse_error = false;
        }
        self.builder
            .add_node(kind, to_u32(pos), to_u32(end), flags, data)
    }

    fn new_node_list(&mut self, pos: usize, end: usize, nodes: Vec<NodeId>) -> NodeList {
        self.builder.add_list(to_u32(pos), to_u32(end), nodes)
    }

    /// Records the `JSDoc` facts for `node`. TypeScript files parse `JSDoc` lazily.
    fn with_jsdoc(&mut self, node: NodeId, info: JsdocScannerInfo) {
        if !info.has_jsdoc {
            return;
        }
        let mut flags = NodeFlags::HAS_JSDOC;
        if info.has_deprecated {
            flags |= NodeFlags::POSSIBLY_CONTAINS_DEPRECATED_TAG;
        }
        self.builder.add_flags(node, flags);
    }

    // Context flags

    fn in_context(&self, flags: NodeFlags) -> bool {
        self.context_flags.intersects(flags)
    }

    fn set_context_flags(&mut self, flags: NodeFlags, value: bool) {
        self.context_flags = if value {
            self.context_flags | flags
        } else {
            self.context_flags.without(flags)
        };
    }

    /// Runs `parse` with `flags` set to `value`, restoring the context flags afterwards.
    fn do_in_context<T>(
        &mut self,
        flags: NodeFlags,
        value: bool,
        parse: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let saved = self.context_flags;
        self.set_context_flags(flags, value);
        let result = parse(self);
        self.context_flags = saved;
        result
    }
}

/// Returns the source text of a punctuation or keyword token.
fn token_to_string(kind: SyntaxKind) -> &'static str {
    crate::scanner::token_to_string(kind).unwrap_or_default()
}

fn to_u32(offset: usize) -> u32 {
    u32::try_from(offset).expect("source offsets fit in u32")
}
