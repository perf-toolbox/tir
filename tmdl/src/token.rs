use std::ops::{Bound, Range, RangeBounds};

use lpl::{ParseStream, Span, Spanned};

/// A token in the TMDL language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token<'src> {
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
    Identifier(&'src str),
    StringLiteral(&'src str),
    IntegerLiteral(i64),
    BitLiteral(i64, i8),

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
    /// `.`
    Dot,
    /// `$`
    Dollar,
    /// `@`
    At,
    /// `"`
    DoubleQuote,

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

    Comment(&'src str),

    // Special
    EOF,
}

#[derive(Debug, Clone)]
pub struct TokenStream<'a, 'src>
where
    'src: 'a,
{
    tokens: &'a [Spanned<Token<'src>>],
    position: usize,
}

impl<'a, 'src> TokenStream<'a, 'src>
where
    'src: 'a,
{
    pub fn new(tokens: &'a [Spanned<Token<'src>>]) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }
}

impl<'a, 'src> ParseStream<'src> for TokenStream<'a, 'src> {
    type Slice = &'a [Spanned<Token<'src>>];
    type Extra = ();
    type Item = Spanned<Token<'src>>;

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
        if self.tokens.is_empty() {
            return Span::new(None, 0, None);
        }
        self.tokens[0].1.clone()
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
