use std::collections::HashMap;

use ariadne::Color;
use ariadne::Label;
use ariadne::Report;
use ariadne::ReportKind;
use thiserror::Error;
use winnow::ascii::alpha1;
use winnow::ascii::alphanumeric0;
use winnow::ascii::line_ending;
use winnow::ascii::multispace0;
use winnow::ascii::space0;
use winnow::combinator::alt;
use winnow::combinator::preceded;
use winnow::combinator::repeat;
use winnow::combinator::repeat_till;
use winnow::combinator::separated;
use winnow::combinator::separated_pair;
use winnow::combinator::terminated;
use winnow::combinator::trace;
use winnow::error::ErrMode;
use winnow::error::ErrorKind;
use winnow::error::ParserError;
use winnow::stream::Stateful;
use winnow::stream::Stream;
use winnow::token::take_till;
use winnow::Parser;

use crate::Attr;
use crate::Block;
use crate::BlockRef;
use crate::Region;
use crate::RegionRef;
use crate::{ContextRef, OpRef, Type};

#[derive(Debug, Clone)]
pub struct ParserState {
    context: ContextRef,
    deferred_type_list: Vec<Type>,
    deferred_arg_names: Vec<String>,
    cur_region: Vec<RegionRef>,
}

impl ParserState {
    pub fn new(context: ContextRef) -> Self {
        ParserState {
            context,
            deferred_type_list: vec![],
            deferred_arg_names: vec![],
            cur_region: vec![],
        }
    }

    pub fn get_context(&self) -> ContextRef {
        self.context.clone()
    }

    pub fn push_region(&mut self, region: RegionRef) {
        self.cur_region.push(region);
    }

    pub fn get_region(&self) -> RegionRef {
        self.cur_region.last().cloned().unwrap()
    }

    pub fn pop_region(&mut self) {
        self.cur_region.pop();
    }

    pub fn take_deferred_types(&mut self) -> Vec<Type> {
        std::mem::take(&mut self.deferred_type_list)
    }

    pub fn set_deferred_types(&mut self, types: Vec<Type>) {
        self.deferred_type_list = types;
    }

    pub fn take_deferred_names(&mut self) -> Vec<String> {
        std::mem::take(&mut self.deferred_arg_names)
    }

    pub fn set_deferred_names(&mut self, names: Vec<String>) {
        self.deferred_arg_names = names;
    }
}

pub type ParseStream<'a> = Stateful<&'a str, ParserState>;

pub type PResult<I> = winnow::PResult<I, PError>;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum PError {
    #[error("unknown dialect '{0}'")]
    UnknownDialect(String),
    #[error("unknown operation '{1}' in dialect '{0}'")]
    UnknownOperation(String, String),
    #[error("expected {0} got {1}")]
    UnexpectedToken(String, String),
    #[error("syntax error")]
    Unknown,
}

impl<I: Stream + Clone> ParserError<I> for PError {
    fn from_error_kind(_input: &I, _kind: ErrorKind) -> Self {
        PError::Unknown
    }

    fn append(self, _: &I, _: &<I as Stream>::Checkpoint, _: ErrorKind) -> Self {
        self
    }
}

pub trait Parseable<T> {
    fn parse(input: &mut ParseStream<'_>) -> PResult<T>;
}

pub fn identifier<'s>(input: &mut ParseStream<'s>) -> PResult<&'s str> {
    (alpha1, alphanumeric0).recognize().parse_next(input)
}

fn dialect_op<'s>(input: &mut ParseStream<'s>) -> PResult<(&'s str, &'s str)> {
    separated_pair(identifier, ".", identifier).parse_next(input)
}

fn builtin_op<'s>(input: &mut ParseStream<'s>) -> PResult<(&'s str, &'s str)> {
    identifier
        .recognize()
        .parse_next(input)
        .map(|op| ("builtin", op))
}

pub fn op_tuple<'s>(input: &mut ParseStream<'s>) -> PResult<(&'s str, &'s str)> {
    alt((dialect_op, builtin_op)).parse_next(input)
}

pub fn sym_name<'s>(input: &mut ParseStream<'s>) -> PResult<&'s str> {
    preceded("@", identifier).parse_next(input)
}

fn comment(input: &mut ParseStream<'_>) -> PResult<()> {
    trace(
        "comment",
        repeat(
            0..,
            (multispace0, ';', take_till(1.., ['\n', '\r']), line_ending).void(),
        ),
    )
    .parse_next(input)
}

pub fn single_op(input: &mut ParseStream) -> PResult<OpRef> {
    let context = input.state.get_context();

    // TODO: find a smarter way
    multispace0.parse_next(input)?;
    comment.parse_next(input)?;
    multispace0.parse_next(input)?;
    let (dialect_name, op_name) = trace("op name", op_tuple).parse_next(input)?;

    let dialect = context
        .get_dialect_by_name(dialect_name)
        .ok_or(ErrMode::Cut(PError::UnknownDialect(
            dialect_name.to_owned(),
        )))?;

    let operation_id =
        dialect
            .get_operation_id(op_name)
            .ok_or(ErrMode::Cut(PError::UnknownOperation(
                dialect_name.to_owned(),
                op_name.to_owned(),
            )))?;

    // It is impossible to add an operation without specifying its parser
    let parser = dialect.get_operation_parser(operation_id).unwrap();
    trace("op body", parser).parse_next(input)
}

pub fn parse_ir(
    context: ContextRef,
    input: &str,
) -> Result<OpRef, winnow::error::ParseError<ParseStream<'_>, PError>> {
    let input = ParseStream {
        input,
        state: ParserState::new(context),
    };

    single_op.parse(input)
}

pub fn single_block_region(ir: &mut ParseStream<'_>) -> PResult<Vec<OpRef>> {
    let _ = (multispace0, "{", multispace0).parse_next(ir)?;

    let operations = repeat(0.., single_op).parse_next(ir)?;

    let _ = (multispace0, "}", multispace0).parse_next(ir)?;

    Ok(operations)
}

pub fn single_block(input: &mut ParseStream<'_>) -> PResult<BlockRef> {
    let block_name = preceded("^", identifier).parse_next(input)?;

    (":", multispace0).parse_next(input)?;

    let ops: Vec<OpRef> = repeat(0.., single_op).parse_next(input)?;

    let region = input.state.get_region();

    let names = input.state.take_deferred_names();
    let types = input.state.take_deferred_types();
    let block = Block::with_arguments(block_name, &region, &types, &names);

    for op in ops {
        block.push(&op);
    }

    Ok(block)
}

pub fn region_with_blocks(input: &mut ParseStream<'_>) -> PResult<RegionRef> {
    (multispace0, "{", multispace0).parse_next(input)?;
    let context = input.state.get_context();
    let region = Region::empty(&context);
    input.state.push_region(region.clone());

    let (blocks, (_, _)): (Vec<BlockRef>, (_, _)) =
        repeat_till(1.., single_block, (multispace0, "}")).parse_next(input)?;

    for block in blocks {
        region.add_block(block);
    }

    input.state.pop_region();

    Ok(region)
}

fn attr_pair(input: &mut ParseStream<'_>) -> PResult<(String, Attr)> {
    separated_pair(
        identifier.map(|s| s.to_string()),
        (space0, "=", space0),
        Attr::parse,
    )
    .parse_next(input)
}

pub fn attr_list(input: &mut ParseStream<'_>) -> PResult<HashMap<String, Attr>> {
    let attr_pairs = separated::<_, _, HashMap<_, _>, _, _, _, _>(
        0..,
        attr_pair,
        (space0, ",", space0).recognize(),
    );
    terminated(
        preceded((space0, "attrs", space0, "=", space0, "{"), attr_pairs),
        (space0, "}", space0),
    )
    .parse_next(input)
}

pub fn print_parser_diag(
    context: ContextRef,
    diag: &winnow::error::ParseError<ParseStream<'_>, PError>,
) {
    let offset = diag.offset();
    let inner = diag.inner();

    let mut builder = Report::<std::ops::Range<usize>>::build(ReportKind::Error, (), offset)
        .with_message(&format!("{}", &inner));

    match inner {
        PError::UnknownOperation(dialect_name, op_name) => {
            builder.add_label(
                Label::new((offset - op_name.len())..offset)
                    .with_color(Color::Red)
                    .with_message("here"),
            );
            if let Some(dialect) = context.get_dialect_by_name(dialect_name) {
                if let Some(name) = dialect.get_similarly_named_op(op_name) {
                    builder.set_note(format!(
                        "There is a similarly named operation '{}' in '{}' dialect",
                        name, &dialect_name
                    ));
                }
            }
        }
        _ => {}
    }

    builder
        .finish()
        .print(ariadne::Source::from(diag.input().input))
        .unwrap();
}

#[cfg(test)]
mod tests {
    use winnow::Parser;

    use super::{identifier, op_tuple, ParseStream, ParserState};
    use crate::Context;

    macro_rules! input {
        ($inp:literal, $context:expr) => {
            ParseStream {
                input: $inp.into(),
                state: ParserState::new($context),
            }
        };
    }

    #[test]
    fn parse_ident() {
        let context = Context::new();
        assert!(identifier.parse(input!("abc", context.clone())).is_ok());
        assert!(identifier.parse(input!("abc123", context.clone())).is_ok());
        assert!(identifier.parse(input!("123", context.clone())).is_err());
        assert!(identifier.parse(input!("123abs", context.clone())).is_err());
        let mut inp = input!("abc123 abc 123", context.clone());
        let ident = identifier.parse_next(&mut inp).unwrap();
        assert_eq!(ident, "abc123");
    }

    #[test]
    fn parse_op_name() {
        let context = Context::new();
        let mut ir = input!("module", context.clone());
        let result = op_tuple.parse_next(&mut ir).unwrap();
        assert_eq!(result, ("builtin", "module"));

        let mut ir = input!("test.module", context.clone());
        let result = op_tuple.parse_next(&mut ir).unwrap();
        assert_eq!(result, ("test", "module"));
    }
}
