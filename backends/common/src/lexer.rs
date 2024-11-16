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
    Diagnostic, ParseStream, Parser, Spanned, StrStream,
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

    fn get<R>(&self, range: R) -> Option<Self::Slice>
    where
        R: RangeBounds<usize>,
    {
        let ub = match range.end_bound() {
            Bound::Included(value) => *value + 1,
            Bound::Excluded(value) => *value,
            Bound::Unbounded => self.tokens.len(),
        };

        if ub <= self.tokens.len() {
            Some(&self.tokens[(range.start_bound().cloned(), range.end_bound().cloned())])
        } else {
            None
        }
    }

    fn slice<R>(&self, range: R) -> Option<Self>
    where
        R: RangeBounds<usize>,
    {
        let ub = match range.end_bound() {
            Bound::Included(value) => *value + 1,
            Bound::Excluded(value) => *value,
            Bound::Unbounded => self.tokens.len(),
        };

        let tokens = &self.tokens[(range.start_bound().cloned(), range.end_bound().cloned())];
        if !tokens.is_empty() {
            Some(Self {
                tokens,
                extra: self.extra.clone(),
            })
        } else {
            None
        }
        // if ub <= self.tokens.len() {
        //     Some(Self {
        //         tokens: &self.tokens[(range.start_bound().cloned(), range.end_bound().cloned())],
        //         extra: self.extra.clone(),
        //     })
        // } else {
        //     None
        // }
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

    fn nth(&self, n: usize) -> Option<Self::Item> {
        self.tokens.get(n).map(|t| t.0.clone())
    }
}

fn allowed_ident_char(c: char) -> bool {
    c == '_'
}

fn directive<'a>() -> impl Parser<'a, StrStream<'a>, AsmToken<'a>> {
    literal(".")
        .and_then(ident(allowed_ident_char))
        .map(|(_, ident_str)| AsmToken::Directive(ident_str))
        .label("asm_directive")
}

fn label<'a>() -> impl Parser<'a, StrStream<'a>, AsmToken<'a>> {
    ident(allowed_ident_char)
        .and_then(literal(":"))
        .map(|(ident_str, _)| AsmToken::Label(ident_str))
        .label("asm_label")
}

fn punct<'a>() -> impl Parser<'a, StrStream<'a>, AsmToken<'a>> {
    literal("(")
        .map(|_| AsmToken::OpenParen)
        .or_else(literal(")").map(|_| AsmToken::CloseParen))
        .or_else(literal(",").map(|_| AsmToken::Comma))
        .label("asm_punct")
}

pub fn lex_asm(input: &str) -> Result<Vec<Spanned<AsmToken>>, Diagnostic> {
    let stream: StrStream = input.into();

    let token = directive()
        .or_else(label())
        .or_else(ident(allowed_ident_char).map(AsmToken::Ident))
        .or_else(dec_number().map(AsmToken::Number))
        .or_else(punct())
        .label("asm_token");

    let lexer = interleaved(token.spanned(), any_whitespace1());

    lexer.parse(stream).map(|(tokens, _)| tokens)
}
