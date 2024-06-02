use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::Enumerate;
use std::ops::Range;
use std::rc::Rc;
use tir_core::OpBuilder;
use winnow::ascii::{alpha1, alphanumeric0, line_ending, multispace0, space1};
use winnow::combinator::{alt, delimited, empty, fail, repeat, terminated};
use winnow::combinator::{dispatch, trace};
use winnow::error::ContextError;
use winnow::stream::{ContainsToken, Offset, Stream, StreamIsPartial};
use winnow::token::{one_of, take, take_till, take_while};
use winnow::{
    ascii::alphanumeric1,
    combinator::{opt, preceded},
    PResult,
};
use winnow::{Located, Parser};

use crate::target::SectionOp;

#[derive(Debug, Clone, PartialEq)]
pub enum AsmToken<'a> {
    Ident(&'a str),
    Section(&'a str),
    Label(&'a str),
    Comma,
}

impl<'a> ContainsToken<AsmToken<'a>> for AsmToken<'a> {
    fn contains_token(&self, token: AsmToken<'_>) -> bool {
        *self == token
    }
}

impl<'a> ContainsToken<AsmToken<'a>> for &'_ [AsmToken<'a>] {
    fn contains_token(&self, token: AsmToken<'a>) -> bool {
        self.iter().any(|t| *t == token)
    }
}

pub type Spanned<'a> = (AsmToken<'a>, Range<usize>);

#[derive(Debug, Clone)]
pub struct TokenStream<'tok, 'src> {
    source: &'tok [Spanned<'src>],
    offset: usize,
    builder: OpBuilder,
    sections: HashMap<String, Rc<RefCell<SectionOp>>>,
    active_section: Option<Rc<RefCell<SectionOp>>>,
}

impl<'tok, 'src> TokenStream<'tok, 'src> {
    pub fn new(builder: &OpBuilder, source: &'tok [Spanned<'src>]) -> Self {
        Self {
            source,
            offset: 0,
            builder: builder.clone(),
            sections: HashMap::new(),
            active_section: None,
        }
    }

    pub fn get_builder(&self) -> OpBuilder {
        self.builder.clone()
    }

    pub fn get_section(&self, name: &str) -> Option<Rc<RefCell<SectionOp>>> {
        self.sections.get(name).cloned()
    }

    pub fn add_section(&mut self, name: &str, section: &Rc<RefCell<SectionOp>>) {
        self.sections.insert(name.to_owned(), section.clone());
    }

    pub fn get_active_section(&self) -> Option<Rc<RefCell<SectionOp>>> {
        self.active_section.clone()
    }

    pub fn set_active_section(&mut self, section: Rc<RefCell<SectionOp>>) {
        self.active_section = Some(section);
    }
}

impl<'tok, 'src> Offset for TokenStream<'tok, 'src> {
    fn offset_from(&self, start: &Self) -> usize {
        self.offset + start.offset
    }
}

pub struct TokenStreamIterator<'stream, 'tok, 'src> {
    stream: &'stream TokenStream<'tok, 'src>,
    offset: usize,
}

impl<'stream, 'tok, 'src> Iterator for TokenStreamIterator<'stream, 'tok, 'src> {
    type Item = AsmToken<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        let tok = self.stream.source.get(self.offset).cloned().map(|t| t.0);
        self.offset += 1;
        tok
    }
}

#[derive(Clone, Debug)]
pub struct Checkpoint {
    offset: usize,
}

impl Offset for Checkpoint {
    fn offset_from(&self, start: &Self) -> usize {
        self.offset + start.offset
    }
}

impl Checkpoint {
    pub fn new(offset: usize) -> Self {
        Self { offset }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl<'tok, 'src> Offset<Checkpoint> for TokenStream<'tok, 'src> {
    fn offset_from(&self, start: &Checkpoint) -> usize {
        self.offset + start.offset()
    }
}

impl<'tok, 'src> Stream for TokenStream<'tok, 'src> {
    type Token = AsmToken<'src>;
    type Slice = &'tok [Spanned<'src>];
    type IterOffsets = Enumerate<TokenStreamIterator<'tok, 'tok, 'src>>;
    type Checkpoint = Checkpoint;

    fn iter_offsets(&self) -> Self::IterOffsets {
        todo!()
    }

    fn eof_offset(&self) -> usize {
        self.source.len() - self.offset
    }

    fn next_token(&mut self) -> Option<Self::Token> {
        let token = self.source.get(self.offset).cloned().map(|t| t.0);
        self.offset += 1;
        token
    }

    fn offset_for<P>(&self, _predicate: P) -> Option<usize>
    where
        P: Fn(Self::Token) -> bool,
    {
        unimplemented!();
    }

    fn offset_at(&self, _tokens: usize) -> Result<usize, winnow::error::Needed> {
        unimplemented!();
    }

    fn next_slice(&mut self, offset: usize) -> Self::Slice {
        self.source.next_slice(offset)
    }

    fn checkpoint(&self) -> Self::Checkpoint {
        Checkpoint::new(self.offset)
    }

    fn reset(&mut self, checkpoint: &Self::Checkpoint) {
        self.offset = checkpoint.offset();
    }

    fn raw(&self) -> &dyn std::fmt::Debug {
        &self.source
    }
}

impl<'tok, 'str> StreamIsPartial for TokenStream<'tok, 'str> {
    type PartialState = ();

    fn complete(&mut self) -> Self::PartialState {
        unreachable!()
    }

    fn restore_partial(&mut self, _state: Self::PartialState) {
        unreachable!()
    }

    fn is_partial_supported() -> bool {
        false
    }
}

/// Shortcut for well-known sections like `.text`. These do not need a `.section` prefix.
fn known_section<'a>(input: &mut Located<&'a str>) -> PResult<&'a str> {
    trace(
        "known section",
        preceded(multispace0, alt((".text", ".data", ".rodata", ".bss"))),
    )
    .parse_next(input)
}

/// A generic section in the format `.section <name>\n`
fn section<'a>(input: &mut Located<&'a str>) -> PResult<AsmToken<'a>> {
    trace(
        "generic section",
        alt((
            known_section,
            preceded(
                (multispace0, ".section", space1),
                (opt("."), alphanumeric1).recognize(),
            ),
        )),
    )
    .map(AsmToken::Section)
    .parse_next(input)
}

/// A basic block label in the format `<name>:`
fn label<'a>(input: &mut Located<&'a str>) -> PResult<AsmToken<'a>> {
    trace(
        "basic block label",
        terminated((alpha1, alphanumeric0).recognize(), ':'),
    )
    .map(AsmToken::Label)
    .parse_next(input)
}

/// Any other identifier
fn ident<'a>(input: &mut Located<&'a str>) -> PResult<AsmToken<'a>> {
    trace(
        "generic identifier",
        (
            alpha1,
            take_while(0.., |c: char| c.is_alphanumeric() || c == '.' || c == '_'),
        ),
    )
    .recognize()
    .map(AsmToken::Ident)
    .parse_next(input)
}

/// Punctuation sign, currently only `,`
fn punct<'a>(input: &mut Located<&'a str>) -> PResult<AsmToken<'a>> {
    let mut chr = delimited(multispace0, take(1_usize), multispace0);

    dispatch! {chr;
        "," => empty.value(AsmToken::Comma),
        _ => fail::<_, AsmToken, _>,
    }
    .parse_next(input)
}

fn single_comment(input: &mut Located<&str>) -> PResult<()> {
    trace(
        "single comment",
        (
            one_of([';', '#']),
            take_till(1.., ['\n', '\r']),
            line_ending,
        ),
    )
    .void()
    .parse_next(input)
}

/// Parse assembly single-line comment starting with `;` or `#`
pub fn comment(input: &mut Located<&str>) -> PResult<()> {
    trace(
        "multi comment",
        repeat(0.., delimited(multispace0, single_comment, multispace0)),
    )
    .parse_next(input)?;
    Ok(())
}

/// Common parser for any token kind
fn token<'a>(input: &mut Located<&'a str>) -> PResult<Spanned<'a>> {
    delimited(comment, alt((section, label, ident, punct)), multispace0)
        .with_span()
        .parse_next(input)
}

/// Split input assembly string into tokens
pub fn lex_asm(
    input: &str,
) -> Result<Vec<Spanned<'_>>, winnow::error::ParseError<Located<&str>, ContextError>> {
    let input = Located::new(input);

    terminated(repeat(0.., token), comment).parse(input)
}

#[cfg(test)]
mod tests {
    use winnow::{Located, Parser};

    use super::{label, section, AsmToken};

    #[test]
    fn sections() {
        let res = section
            .parse(Located::new(".section .text"))
            .expect("section");
        match res {
            AsmToken::Section(name) => assert_eq!(name, ".text"),
            _ => panic!("Not a section"),
        };

        let res = section.parse(Located::new(".text")).expect("section");
        match res {
            AsmToken::Section(name) => assert_eq!(name, ".text"),
            _ => panic!("Not a section"),
        };
    }

    #[test]
    fn labels() {
        let res = label.parse(Located::new("foo:")).expect("label");
        match res {
            AsmToken::Label(name) => assert_eq!(name, "foo"),
            _ => panic!("Not a section"),
        };
        assert!(label.parse(Located::new("1bar:")).is_err());
    }
}
