//! A dependency-free lexical scanner for the TypeScript front end.

use crate::source_text::Utf16Offset;
use crate::syntax::{Diagnostic, TextSpan};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TokenKind {
    Const,
    Let,
    Var,
    Interface,
    Enum,
    Namespace,
    Class,
    Extends,
    New,
    Type,
    Function,
    Return,
    Throw,
    For,
    Do,
    While,
    Switch,
    Try,
    Catch,
    Finally,
    Break,
    Continue,
    Export,
    Default,
    Import,
    Identifier(String),
    NumberLiteral(String),
    BooleanLiteral(bool),
    NullLiteral,
    StringLiteral(String),
    Colon,
    Arrow,
    Equals,
    EqualsEquals,
    EqualsEqualsEquals,
    Semicolon,
    Comma,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    LeftParen,
    RightParen,
    Dot,
    Question,
    QuestionQuestion,
    Bang,
    BangEquals,
    BangEqualsEquals,
    AmpersandAmpersand,
    Plus,
    PlusEquals,
    Minus,
    Asterisk,
    Slash,
    Percent,
    Pipe,
    PipePipe,
    LessThan,
    LessThanEquals,
    GreaterThan,
    GreaterThanEquals,
    Unknown(char),
    EndOfFile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Token {
    pub(super) kind: TokenKind,
    pub(super) span: TextSpan,
}

pub(super) fn scan(text: &str) -> (Vec<Token>, Vec<Diagnostic>) {
    Scanner::new(text).scan_tokens()
}

struct Scanner<'source> {
    text: &'source str,
    byte_offset: usize,
    utf16_offset: usize,
    diagnostics: Vec<Diagnostic>,
}

impl<'source> Scanner<'source> {
    fn new(text: &'source str) -> Self {
        Self {
            text,
            byte_offset: 0,
            utf16_offset: 0,
            diagnostics: Vec::new(),
        }
    }

    fn scan_tokens(mut self) -> (Vec<Token>, Vec<Diagnostic>) {
        let mut tokens = Vec::new();
        while let Some(character) = self.peek() {
            if character.is_whitespace() {
                self.advance();
                continue;
            }

            if self.starts_with("//") {
                self.skip_line_comment();
                continue;
            }

            if self.starts_with("/*") {
                self.skip_block_comment();
                continue;
            }

            let token = self.scan_token();
            if matches!(token.kind, TokenKind::Unknown(_)) {
                self.diagnostics
                    .push(Diagnostic::new(1127, "Invalid character.", token.span));
            }
            tokens.push(token);
        }

        tokens.push(Token {
            kind: TokenKind::EndOfFile,
            span: TextSpan::new(Utf16Offset::new(self.utf16_offset), 0),
        });
        (tokens, self.diagnostics)
    }

    fn scan_token(&mut self) -> Token {
        let start_byte = self.byte_offset;
        let start = self.utf16_offset;
        let character = self
            .advance()
            .expect("scanner is positioned before end of input");
        let kind = match character {
            '$' | '_' if self.peek().is_some_and(is_identifier_part) => {
                self.scan_identifier(start_byte)
            }
            '$' | '_' => TokenKind::Identifier(character.to_string()),
            character if is_identifier_start(character) => self.scan_identifier(start_byte),
            '0'..='9' => self.scan_number(start_byte),
            '\'' | '"' | '`' => self.scan_string(character, start_byte, start),
            ':' => TokenKind::Colon,
            '=' if self.matches_next('>') => TokenKind::Arrow,
            '=' if self.matches_next('=') => {
                if self.matches_next('=') {
                    TokenKind::EqualsEqualsEquals
                } else {
                    TokenKind::EqualsEquals
                }
            }
            '=' => TokenKind::Equals,
            ';' => TokenKind::Semicolon,
            ',' => TokenKind::Comma,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '.' if self.peek().is_some_and(|next| next.is_ascii_digit()) => {
                self.scan_number(start_byte)
            }
            '.' => TokenKind::Dot,
            '?' if self.matches_next('?') => TokenKind::QuestionQuestion,
            '?' => TokenKind::Question,
            '!' if self.matches_next('=') => {
                if self.matches_next('=') {
                    TokenKind::BangEqualsEquals
                } else {
                    TokenKind::BangEquals
                }
            }
            '!' => TokenKind::Bang,
            '&' if self.matches_next('&') => TokenKind::AmpersandAmpersand,
            '+' if self.matches_next('=') => TokenKind::PlusEquals,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Asterisk,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '|' if self.matches_next('|') => TokenKind::PipePipe,
            '|' => TokenKind::Pipe,
            '<' if self.matches_next('=') => TokenKind::LessThanEquals,
            '<' => TokenKind::LessThan,
            '>' if self.matches_next('=') => TokenKind::GreaterThanEquals,
            '>' => TokenKind::GreaterThan,
            _ => TokenKind::Unknown(character),
        };

        Token {
            kind,
            span: TextSpan::new(Utf16Offset::new(start), self.utf16_offset - start),
        }
    }

    fn scan_identifier(&mut self, start_byte: usize) -> TokenKind {
        while self.peek().is_some_and(is_identifier_part) {
            self.advance();
        }

        let value = &self.text[start_byte..self.byte_offset];
        match value {
            "const" => TokenKind::Const,
            "let" => TokenKind::Let,
            "var" => TokenKind::Var,
            "interface" => TokenKind::Interface,
            "enum" => TokenKind::Enum,
            "namespace" => TokenKind::Namespace,
            "class" => TokenKind::Class,
            "extends" => TokenKind::Extends,
            "new" => TokenKind::New,
            "type" => TokenKind::Type,
            "function" => TokenKind::Function,
            "return" => TokenKind::Return,
            "throw" => TokenKind::Throw,
            "for" => TokenKind::For,
            "do" => TokenKind::Do,
            "while" => TokenKind::While,
            "switch" => TokenKind::Switch,
            "try" => TokenKind::Try,
            "catch" => TokenKind::Catch,
            "finally" => TokenKind::Finally,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "export" => TokenKind::Export,
            "default" => TokenKind::Default,
            "import" => TokenKind::Import,
            "true" => TokenKind::BooleanLiteral(true),
            "false" => TokenKind::BooleanLiteral(false),
            "null" => TokenKind::NullLiteral,
            _ => TokenKind::Identifier(value.to_owned()),
        }
    }

    fn scan_number(&mut self, start_byte: usize) -> TokenKind {
        let source = &self.text[start_byte..];
        let radix = if source.starts_with("0x") || source.starts_with("0X") {
            Some(16)
        } else if source.starts_with("0b") || source.starts_with("0B") {
            Some(2)
        } else if source.starts_with("0o") || source.starts_with("0O") {
            Some(8)
        } else {
            None
        };

        if let Some(radix) = radix {
            self.advance();
            while self
                .peek()
                .is_some_and(|character| character == '_' || character.is_digit(radix))
            {
                self.advance();
            }
            if self.peek() == Some('n') {
                self.advance();
            }
        } else {
            while self
                .peek()
                .is_some_and(|character| character.is_ascii_digit() || character == '_')
            {
                self.advance();
            }
            if self.peek() == Some('.') && !self.starts_with("..") {
                self.advance();
                while self
                    .peek()
                    .is_some_and(|character| character.is_ascii_digit() || character == '_')
                {
                    self.advance();
                }
            }
            if self
                .peek()
                .is_some_and(|character| matches!(character, 'e' | 'E'))
            {
                self.advance();
                if self
                    .peek()
                    .is_some_and(|character| matches!(character, '+' | '-'))
                {
                    self.advance();
                }
                while self
                    .peek()
                    .is_some_and(|character| character.is_ascii_digit() || character == '_')
                {
                    self.advance();
                }
            }
            if self.peek() == Some('n') {
                self.advance();
            }
        }

        TokenKind::NumberLiteral(self.text[start_byte..self.byte_offset].to_owned())
    }

    fn scan_string(&mut self, quote: char, start_byte: usize, start_utf16: usize) -> TokenKind {
        let mut terminated = false;
        while let Some(character) = self.peek() {
            self.advance();
            if character == quote {
                terminated = true;
                break;
            }
            if character == '\\' {
                self.advance();
            }
        }
        if !terminated {
            self.diagnostics.push(Diagnostic::new(
                1002,
                "unterminated string literal",
                TextSpan::new(
                    Utf16Offset::new(start_utf16),
                    self.utf16_offset - start_utf16,
                ),
            ));
        }
        TokenKind::StringLiteral(self.text[start_byte..self.byte_offset].to_owned())
    }

    fn skip_line_comment(&mut self) {
        self.advance();
        self.advance();
        while self
            .peek()
            .is_some_and(|character| !is_line_terminator(character))
        {
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) {
        self.advance();
        self.advance();
        while self.peek().is_some() && !self.starts_with("*/") {
            self.advance();
        }
        if self.starts_with("*/") {
            self.advance();
            self.advance();
        }
    }

    fn starts_with(&self, value: &str) -> bool {
        self.text[self.byte_offset..].starts_with(value)
    }

    fn matches_next(&mut self, expected: char) -> bool {
        if self.peek() != Some(expected) {
            return false;
        }
        self.advance();
        true
    }

    fn peek(&self) -> Option<char> {
        self.text[self.byte_offset..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let character = self.peek()?;
        self.byte_offset += character.len_utf8();
        self.utf16_offset += character.len_utf16();
        Some(character)
    }
}

fn is_identifier_start(character: char) -> bool {
    character == '$' || character == '_' || character.is_alphabetic()
}

fn is_identifier_part(character: char) -> bool {
    is_identifier_start(character) || character.is_numeric()
}

fn is_line_terminator(character: char) -> bool {
    matches!(character, '\r' | '\n' | '\u{2028}' | '\u{2029}')
}
