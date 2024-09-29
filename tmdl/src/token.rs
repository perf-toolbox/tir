use std::ops::{Bound, Range, RangeBounds};

use lpl::{ParseStream, Span, Spanned};

/// A token in the TMDL language.
#[derive(Debug, Clone)]
pub enum Token<'a> {
    // Keywords
    /// `instr_template`
    InstrTemplate,
    /// `properties`
    Properties,
    /// `for`
    For,
    /// `let`
    Let,
    /// `instr`
    Instr,

    // Identifiers and literals
    Identifier(&'a str),
    StringLiteral(&'a str),
    IntegerLiteral(i64),

    // Symbols
    /// `{`
    LeftBrace,
    /// `}`
    RightBrace,
    /// `<`
    LeftAngle,
    /// `>`
    RightAngle,
    /// `:`
    Colon,
    /// `;`
    Semicolon,
    /// `,`
    Comma,
    /// `$`
    Dollar,
    /// `@`
    At,

    // Types
    /// `Register`
    Register,
    /// `bits`
    Bits,
    /// `str`
    Str,

    // Operators
    /// `=`
    Equals,

    Comment(&'a str),

    // Special
    EOF,
}

#[derive(Debug, Clone)]
pub struct TokenStream<'a> {
    tokens: &'a [Spanned<Token<'a>>],
    position: usize,
}

impl<'a> TokenStream<'a> {
    pub fn new(tokens: &'a [Spanned<Token<'a>>]) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }
}

impl<'a> ParseStream<'a> for TokenStream<'a> {
    type Slice = &'a [Spanned<Token<'a>>];
    type Extra = ();
    type Item = Spanned<Token<'a>>;

    fn get(&self, range: Range<usize>) -> Option<Self::Slice> {
        let ub = match range.end_bound() {
            Bound::Included(value) => *value + 1,
            Bound::Excluded(value) => *value,
            Bound::Unbounded => self.tokens.len(),
        };

        if ub <= self.tokens.len() {
            Some(&self.tokens[range])
        } else {
            None
        }
    }

    fn slice(&self, range: Range<usize>) -> Option<Self>
    where
        Self: Sized,
    {
        Some(TokenStream::new(&self.tokens[range]))
    }

    fn len(&self) -> usize {
        self.tokens.len()
    }

    fn span(&self) -> Span {
        unimplemented!()
    }

    fn set_extra(&mut self, _extra: Self::Extra) {}
    fn get_extra(&self) -> Option<&Self::Extra> {
        None
    }

    fn peek(&self) -> Option<Self::Item> {
        self.tokens.get(self.position).cloned()
    }

    fn is_string_like(&self) -> bool {
        false
    }
}
