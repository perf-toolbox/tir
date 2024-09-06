#![allow(dependency_on_unit_never_type_fallback)]

use std::{
    cell::RefCell,
    collections::HashMap,
    ops::{Bound, RangeBounds},
    rc::Rc,
};

use lpl::{
    combinators::{
        any_whitespace1, interleaved, literal,
        text::{dec_number, ident},
    },
    ParseStream, Parser, ParserError, Spanned, StrStream,
};
use tir_core::OpBuilder;

use crate::target::SectionOp;

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

#[derive(Debug, Clone)]
pub struct AsmParserContext {
    builder: OpBuilder,
    sections: Rc<RefCell<HashMap<String, Rc<RefCell<SectionOp>>>>>,
    active_section: Rc<RefCell<Option<Rc<RefCell<SectionOp>>>>>,
}

impl AsmParserContext {
    pub fn new(builder: OpBuilder) -> Self {
        Self {
            builder,
            sections: Rc::new(RefCell::new(HashMap::new())),
            active_section: Rc::new(RefCell::new(None)),
        }
    }

    pub fn get_builder(&self) -> OpBuilder {
        self.builder.clone()
    }

    pub fn get_section(&self, name: &str) -> Option<Rc<RefCell<SectionOp>>> {
        self.sections.borrow().get(name).cloned()
    }

    pub fn add_section(&self, name: &str, section: &Rc<RefCell<SectionOp>>) {
        self.sections
            .borrow_mut()
            .insert(name.to_owned(), section.clone());
    }

    pub fn get_active_section(&self) -> Option<Rc<RefCell<SectionOp>>> {
        self.active_section.borrow().clone()
    }

    pub fn set_active_section(&self, section: Rc<RefCell<SectionOp>>) {
        *self.active_section.borrow_mut() = Some(section);
    }
}

#[derive(Clone, Debug)]
pub struct TokenStream<'a> {
    tokens: &'a [Spanned<AsmToken<'a>>],
    extra: AsmParserContext,
}

impl<'a> TokenStream<'a> {
    pub fn new(tokens: &'a [Spanned<AsmToken<'a>>], builder: OpBuilder) -> Self {
        Self {
            tokens,
            extra: AsmParserContext::new(builder),
        }
    }
}

impl<'a> ParseStream<'a> for TokenStream<'a> {
    type Slice = &'a [Spanned<AsmToken<'a>>];
    type Extra = AsmParserContext;
    type Item = AsmToken<'a>;

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
                extra: self.extra.clone(),
            })
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.tokens.len()
    }

    fn span(&self) -> lpl::Span {
        self.tokens.first().unwrap().1.clone()
    }

    fn set_extra(&mut self, extra: Self::Extra) {
        self.extra = extra
    }

    fn get_extra(&self) -> Option<&Self::Extra> {
        Some(&self.extra)
    }

    fn peek(&self) -> Option<Self::Item> {
        self.tokens.first().map(|(t, _)| t).cloned()
    }
}

fn allowed_ident_char(c: char) -> bool {
    c == '_'
}

fn directive<'a>() -> impl Parser<'a, StrStream<'a>, AsmToken<'a>> {
    literal(".")
        .and_then(ident(allowed_ident_char))
        .map(|(_, ident_str)| AsmToken::Directive(ident_str))
}

fn label<'a>() -> impl Parser<'a, StrStream<'a>, AsmToken<'a>> {
    ident(allowed_ident_char)
        .and_then(literal(":"))
        .map(|(ident_str, _)| AsmToken::Label(ident_str))
}

fn punct<'a>() -> impl Parser<'a, StrStream<'a>, AsmToken<'a>> {
    literal("(")
        .map(|_| AsmToken::OpenParen)
        .or_else(literal(")").map(|_| AsmToken::CloseParen))
        .or_else(literal(",").map(|_| AsmToken::Comma))
}

pub fn lex_asm<'a>(input: &'a str) -> Result<Vec<Spanned<AsmToken<'a>>>, ParserError> {
    let stream: StrStream = input.into();

    let token = directive()
        .or_else(label())
        .or_else(ident(allowed_ident_char).map(AsmToken::Ident))
        .or_else(dec_number().map(AsmToken::Number))
        .or_else(punct());

    let lexer = interleaved(token.spanned(), any_whitespace1());

    lexer.parse(stream).map(|(tokens, _)| tokens)
}
