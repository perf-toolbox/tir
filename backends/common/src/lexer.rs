#![allow(dependency_on_unit_never_type_fallback)]

use std::ops::{Bound, RangeBounds};

use lpl::{combinators::{any_whitespace1, interleaved, literal, text::{dec_number, ident}}, ParseStream, Parser, ParserError, Spanned, StrStream};

#[derive(Debug, Clone, PartialEq)]
pub enum AsmToken<'a> {
    Directive(&'a str),
    Ident(&'a str),
    Label(&'a str),
    Comment(&'a str),
    Number(i64),
    OpenParen,
    CloseParen,
    Comma,
}

#[derive(Clone, Debug)]
pub struct TokenStream<'a> {
    tokens: &'a [Spanned<AsmToken<'a>>],
}

impl<'a> TokenStream<'a> {
    pub fn new(tokens: &'a [Spanned<AsmToken<'a>>]) -> Self {
        Self { tokens }
    }
}

impl<'a> ParseStream<'a> for TokenStream<'a> {
    type Slice = &'a [Spanned<AsmToken<'a>>];

    fn get(&self, range: std::ops::Range<usize>) -> Option<Self::Slice> {
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

    fn slice(&self, range: std::ops::Range<usize>) -> Option<Self>
    where
        Self: Sized,
    {
        let ub = match range.end_bound() {
            Bound::Included(value) => *value + 1,
            Bound::Excluded(value) => *value,
            Bound::Unbounded => self.tokens.len(),
        };

        if ub <= self.tokens.len() {
            Some(Self {
                tokens: &self.tokens[range],
            })
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.tokens.len()
    }

    fn span(&self) -> lpl::Span {
        todo!()
    }
}

fn allowed_ident_char(c: char) -> bool {
    c == '_'
}

fn directive<'a>() -> impl Parser<'a, StrStream<'a>, AsmToken<'a>> {
    literal(".").and_then(ident(allowed_ident_char)).map(|(_, ident_str)| AsmToken::Directive(ident_str))
}

fn label<'a>() -> impl Parser<'a, StrStream<'a>, AsmToken<'a>> {
    ident(allowed_ident_char).and_then(literal(":")).map(|(ident_str, _)| AsmToken::Label(ident_str))
}

fn punct<'a>() -> impl Parser<'a, StrStream<'a>, AsmToken<'a>> {
    literal("(").map(|_| AsmToken::OpenParen)
        .or_else(literal(")").map(|_| AsmToken::CloseParen))
        .or_else(literal(",").map(|_| AsmToken::Comma))
}

pub fn lex_asm<'a>(input: &'a str) -> Result<Vec<Spanned<AsmToken<'a>>>, ParserError> {
    let stream: StrStream = input.into();

    let token = directive().or_else(label()).or_else(dec_number().map(AsmToken::Number)).or_else(punct());

    let lexer = interleaved(token.spanned(), any_whitespace1());

    lexer.parse(stream).map(|(tokens, _)| tokens)
}
