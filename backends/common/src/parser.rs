use tir_core::parser::AsmPResult;
use tir_core::OpBuilder;
use winnow::Parser;
use winnow::{
    ascii::{line_ending, multispace0},
    combinator::{preceded, repeat},
    token::{one_of, take_till},
    Stateful,
};

#[derive(Debug, Clone)]
pub struct AsmParserState {
    builder: OpBuilder,
}

impl AsmParserState {
    pub fn new(builder: OpBuilder) -> Self {
        Self { builder }
    }
    pub fn get_builder(&self) -> OpBuilder {
        self.builder.clone()
    }
}

pub type AsmStream<'a> = Stateful<&'a str, AsmParserState>;

fn single_comment(input: &mut AsmStream<'_>) -> AsmPResult<()> {
    (
        one_of([';', '#']),
        take_till(1.., ['\n', '\r']),
        line_ending,
    )
        .void()
        .parse_next(input)
}

pub fn comment(input: &mut AsmStream<'_>) -> AsmPResult<()> {
    repeat(0.., preceded(multispace0, single_comment)).parse_next(input)
}
