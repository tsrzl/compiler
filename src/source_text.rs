//! Immutable source text and TypeScript-compatible source positions.

use std::sync::Arc;

/// An offset measured in UTF-16 code units, as used by the TypeScript compiler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Utf16Offset(usize);

impl Utf16Offset {
    /// Creates an offset from a UTF-16 code-unit count.
    #[must_use]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    /// Returns the UTF-16 code-unit count.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

/// A zero-based line and UTF-16 column pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LinePosition {
    line: usize,
    column: usize,
}

impl LinePosition {
    /// Creates a zero-based line and column pair.
    #[must_use]
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Returns the zero-based line number.
    #[must_use]
    pub const fn line(self) -> usize {
        self.line
    }

    /// Returns the zero-based UTF-16 column.
    #[must_use]
    pub const fn column(self) -> usize {
        self.column
    }
}

/// Immutable UTF-8 source text with a precomputed UTF-16 line map.
#[derive(Debug, Clone)]
pub struct SourceText {
    text: Arc<str>,
    line_starts: Box<[Utf16Offset]>,
    utf16_len: usize,
}

impl SourceText {
    /// Creates source text and indexes the start of each line.
    #[must_use]
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        let text = text.into();
        let (line_starts, utf16_len) = Self::build_line_map(&text);
        Self {
            text,
            line_starts: line_starts.into_boxed_slice(),
            utf16_len,
        }
    }

    /// Returns the original UTF-8 source text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Returns the number of UTF-16 code units in the source text.
    #[must_use]
    pub const fn len_utf16(&self) -> usize {
        self.utf16_len
    }

    /// Maps a UTF-16 source offset to a zero-based line and column.
    ///
    /// Returns `None` when the offset is past the end of the source text.
    #[must_use]
    pub fn line_position(&self, offset: Utf16Offset) -> Option<LinePosition> {
        if offset.get() > self.utf16_len {
            return None;
        }

        let line = self
            .line_starts
            .partition_point(|start| start.get() <= offset.get())
            - 1;
        let column = offset.get() - self.line_starts[line].get();
        Some(LinePosition::new(line, column))
    }

    /// Converts a UTF-16 source offset to a UTF-8 byte offset into [`Self::as_str`].
    ///
    /// Returns `None` when the offset is past the end of the source text or splits a
    /// surrogate pair.
    #[must_use]
    pub fn utf8_offset(&self, offset: Utf16Offset) -> Option<usize> {
        let mut utf16_offset = 0;
        for (byte_offset, character) in self.text.char_indices() {
            if utf16_offset == offset.get() {
                return Some(byte_offset);
            }
            if utf16_offset > offset.get() {
                return None;
            }
            utf16_offset += character.len_utf16();
        }
        (utf16_offset == offset.get()).then_some(self.text.len())
    }

    fn build_line_map(text: &str) -> (Vec<Utf16Offset>, usize) {
        let mut line_starts = vec![Utf16Offset::new(0)];
        let mut utf16_offset = 0;
        let mut characters = text.char_indices().peekable();

        while let Some((_, character)) = characters.next() {
            match character {
                '\r' => {
                    utf16_offset += 1;
                    if characters
                        .peek()
                        .is_some_and(|(_, next_character)| *next_character == '\n')
                    {
                        characters.next();
                        utf16_offset += 1;
                    }
                    line_starts.push(Utf16Offset::new(utf16_offset));
                }
                '\n' | '\u{2028}' | '\u{2029}' => {
                    utf16_offset += 1;
                    line_starts.push(Utf16Offset::new(utf16_offset));
                }
                _ => utf16_offset += character.len_utf16(),
            }
        }

        (line_starts, utf16_offset)
    }
}
