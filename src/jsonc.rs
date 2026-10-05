//! Dependency-free parser for JSON with TypeScript-style comments and trailing commas.

use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum JsonValue {
    Object(BTreeMap<String, Self>),
    Array(Vec<Self>),
    String(String),
    Number,
    Boolean(bool),
    Null,
}

pub(crate) struct JsonParser<'source> {
    source: &'source str,
    offset: usize,
}

impl<'source> JsonParser<'source> {
    pub(crate) fn parse(source: &'source str) -> Result<JsonValue, String> {
        let mut parser = Self { source, offset: 0 };
        parser.skip_trivia()?;
        let value = parser.parse_value()?;
        parser.skip_trivia()?;
        if parser.offset != source.len() {
            return Err(parser.error("unexpected content after the JSON value"));
        }
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_trivia()?;
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string().map(JsonValue::String),
            Some('t') if self.consume_keyword("true") => Ok(JsonValue::Boolean(true)),
            Some('f') if self.consume_keyword("false") => Ok(JsonValue::Boolean(false)),
            Some('n') if self.consume_keyword("null") => Ok(JsonValue::Null),
            Some('-' | '0'..='9') => self.parse_number().map(|()| JsonValue::Number),
            _ => Err(self.error("expected a JSON value")),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.expect('{')?;
        self.skip_trivia()?;
        let mut properties = BTreeMap::new();
        if self.take('}') {
            return Ok(JsonValue::Object(properties));
        }

        loop {
            self.skip_trivia()?;
            if self.peek() != Some('"') {
                return Err(self.error("expected a quoted JSON property name"));
            }
            let name = self.parse_string()?;
            self.skip_trivia()?;
            self.expect(':')?;
            let value = self.parse_value()?;
            properties.insert(name, value);
            self.skip_trivia()?;
            if self.take('}') {
                break;
            }
            self.expect(',')?;
            self.skip_trivia()?;
            if self.take('}') {
                break;
            }
        }
        Ok(JsonValue::Object(properties))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.expect('[')?;
        self.skip_trivia()?;
        let mut elements = Vec::new();
        if self.take(']') {
            return Ok(JsonValue::Array(elements));
        }

        loop {
            elements.push(self.parse_value()?);
            self.skip_trivia()?;
            if self.take(']') {
                break;
            }
            self.expect(',')?;
            self.skip_trivia()?;
            if self.take(']') {
                break;
            }
        }
        Ok(JsonValue::Array(elements))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect('"')?;
        let mut value = String::new();
        loop {
            let Some(character) = self.advance() else {
                return Err(self.error("unterminated JSON string"));
            };
            match character {
                '"' => return Ok(value),
                '\\' => self.parse_escape(&mut value)?,
                character if character < ' ' => {
                    return Err(self.error("control character in JSON string"));
                }
                character => value.push(character),
            }
        }
    }

    fn parse_escape(&mut self, output: &mut String) -> Result<(), String> {
        let Some(escape) = self.advance() else {
            return Err(self.error("unterminated JSON escape"));
        };
        match escape {
            '"' => output.push('"'),
            '\\' => output.push('\\'),
            '/' => output.push('/'),
            'b' => output.push('\u{0008}'),
            'f' => output.push('\u{000c}'),
            'n' => output.push('\n'),
            'r' => output.push('\r'),
            't' => output.push('\t'),
            'u' => self.parse_unicode_escape(output)?,
            _ => return Err(self.error("invalid JSON escape")),
        }
        Ok(())
    }

    fn parse_unicode_escape(&mut self, output: &mut String) -> Result<(), String> {
        let first = self.parse_hex_quad()?;
        let code_point = match first {
            0xd800..=0xdbff if self.consume_literal("\\u") => {
                let second = self.parse_hex_quad()?;
                if !(0xdc00..=0xdfff).contains(&second) {
                    return Err(self.error("invalid low surrogate in JSON escape"));
                }
                0x1_0000 + (u32::from(first - 0xd800) << 10) + u32::from(second - 0xdc00)
            }
            0xd800..=0xdfff => return Err(self.error("unpaired surrogate in JSON escape")),
            value => u32::from(value),
        };
        let Some(character) = char::from_u32(code_point) else {
            return Err(self.error("invalid Unicode scalar value in JSON escape"));
        };
        output.push(character);
        Ok(())
    }

    fn parse_hex_quad(&mut self) -> Result<u16, String> {
        let mut value = 0_u16;
        for _ in 0..4 {
            let Some(character) = self.advance() else {
                return Err(self.error("incomplete Unicode escape in JSON string"));
            };
            let Some(digit) = character.to_digit(16) else {
                return Err(self.error("invalid hexadecimal digit in JSON escape"));
            };
            value = value * 16 + u16::try_from(digit).expect("a hexadecimal digit fits in u16");
        }
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<(), String> {
        self.take('-');
        match self.peek() {
            Some('0') => {
                self.advance();
                if self
                    .peek()
                    .is_some_and(|character| character.is_ascii_digit())
                {
                    return Err(self.error("leading zero in JSON number"));
                }
            }
            Some('1'..='9') => self.consume_digits(),
            _ => return Err(self.error("expected a digit in JSON number")),
        }
        if self.take('.') {
            if !self
                .peek()
                .is_some_and(|character| character.is_ascii_digit())
            {
                return Err(self.error("expected a digit after decimal point"));
            }
            self.consume_digits();
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
            if !self
                .peek()
                .is_some_and(|character| character.is_ascii_digit())
            {
                return Err(self.error("expected a digit in JSON exponent"));
            }
            self.consume_digits();
        }
        Ok(())
    }

    fn consume_digits(&mut self) {
        while self
            .peek()
            .is_some_and(|character| character.is_ascii_digit())
        {
            self.advance();
        }
    }

    fn skip_trivia(&mut self) -> Result<(), String> {
        loop {
            while self.peek().is_some_and(char::is_whitespace) {
                self.advance();
            }
            if self.consume_literal("//") {
                while self
                    .peek()
                    .is_some_and(|character| !matches!(character, '\n' | '\r'))
                {
                    self.advance();
                }
            } else if self.consume_literal("/*") {
                if !self.consume_until("*/") {
                    return Err(self.error("unterminated block comment"));
                }
            } else {
                return Ok(());
            }
        }
    }

    fn consume_until(&mut self, literal: &str) -> bool {
        while self.offset < self.source.len() {
            if self.consume_literal(literal) {
                return true;
            }
            self.advance();
        }
        false
    }

    fn consume_keyword(&mut self, keyword: &str) -> bool {
        if self.source[self.offset..].starts_with(keyword) {
            self.offset += keyword.len();
            true
        } else {
            false
        }
    }

    fn consume_literal(&mut self, literal: &str) -> bool {
        self.consume_keyword(literal)
    }

    fn expect(&mut self, expected: char) -> Result<(), String> {
        if self.take(expected) {
            Ok(())
        } else {
            Err(self.error(&format!("expected '{expected}'")))
        }
    }

    fn take(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let character = self.peek()?;
        self.offset += character.len_utf8();
        Some(character)
    }

    fn error(&self, message: &str) -> String {
        format!("invalid JSONC at byte {}: {message}", self.offset)
    }
}
