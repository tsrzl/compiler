//! TypeScript syntax trees produced by the built-in scanner and parser.

mod ast;
mod parser;
mod scanner;

use crate::source_file::{FileId, SourceFile};
use crate::source_text::Utf16Offset;

pub use ast::{
    ArrowFunctionBody, AssignmentOperator, BinaryOperator, CatchClause, ClassDeclaration,
    ClassMember, EnumDeclaration, EnumMember, ExportAllDeclaration, ExportNamedFromDeclaration,
    ExportSpecifier, Expression, ForInitializer, FunctionBodyStatement, FunctionDeclaration,
    FunctionParameter, ImportDeclaration, ImportSpecifier, InterfaceDeclaration, ObjectProperty,
    Program, PropertyDeclaration, PropertySignature, ReturnStatement, Statement, SwitchClause,
    TypeAliasDeclaration, TypeReference, UnaryOperator, VariableDeclaration,
    VariableDeclarationKind,
};

/// A source range measured in UTF-16 code units.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextSpan {
    start: Utf16Offset,
    length: usize,
}

impl TextSpan {
    /// Creates a span from its UTF-16 start and length.
    #[must_use]
    pub const fn new(start: Utf16Offset, length: usize) -> Self {
        Self { start, length }
    }

    /// Returns the UTF-16 start offset.
    #[must_use]
    pub const fn start(self) -> Utf16Offset {
        self.start
    }

    /// Returns the UTF-16 length.
    #[must_use]
    pub const fn length(self) -> usize {
        self.length
    }
}

/// A parser diagnostic with its stable numeric code and source span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    code: u32,
    message: String,
    span: TextSpan,
}

impl Diagnostic {
    /// Creates a diagnostic with an owned message and UTF-16 source span.
    #[must_use]
    pub fn new(code: u32, message: impl Into<String>, span: TextSpan) -> Self {
        Self {
            code,
            message: message.into(),
            span,
        }
    }

    /// Returns the diagnostic code.
    #[must_use]
    pub const fn code(&self) -> u32 {
        self.code
    }

    /// Returns the diagnostic message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the diagnostic source span.
    #[must_use]
    pub const fn span(&self) -> TextSpan {
        self.span
    }
}

/// A parsed source file, its syntax tree, and parser diagnostics.
#[derive(Debug, Clone)]
pub struct SyntaxTree {
    file_id: FileId,
    source_file: SourceFile,
    program: Program,
    diagnostics: Vec<Diagnostic>,
}

impl SyntaxTree {
    /// Parses a standalone source file as the first file of its own compilation.
    #[must_use]
    pub fn parse(source_file: SourceFile) -> Self {
        Self::parse_file(FileId::new(0), source_file)
    }

    /// Parses a source file with its compilation-scoped identity.
    #[must_use]
    pub fn parse_file(file_id: FileId, source_file: SourceFile) -> Self {
        let (tokens, mut diagnostics) = scanner::scan(source_file.text().as_str());
        let (program, parser_diagnostics) = parser::Parser::new(tokens).parse_program();
        diagnostics.extend(parser_diagnostics);
        diagnostics.sort_by_key(|diagnostic| diagnostic.span().start());
        Self {
            file_id,
            source_file,
            program,
            diagnostics,
        }
    }

    /// Returns the compilation-scoped identity of the parsed file.
    #[must_use]
    pub const fn file_id(&self) -> FileId {
        self.file_id
    }

    /// Returns the source file that produced this tree.
    #[must_use]
    pub const fn source_file(&self) -> &SourceFile {
        &self.source_file
    }

    /// Returns the parsed program.
    #[must_use]
    pub const fn program(&self) -> &Program {
        &self.program
    }

    /// Returns parser diagnostics in source order.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}
