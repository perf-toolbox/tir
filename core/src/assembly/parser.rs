use std::collections::HashMap;

use ariadne::Color;
use ariadne::Config;
use ariadne::Label;
use ariadne::Report;
use ariadne::ReportKind;
use thiserror::Error;
use winnow::ascii::alpha1;
use winnow::ascii::line_ending;
use winnow::ascii::multispace0;
use winnow::ascii::space0;
use winnow::combinator::alt;
use winnow::combinator::cut_err;
use winnow::combinator::delimited;
use winnow::combinator::preceded;
use winnow::combinator::repeat;
use winnow::combinator::repeat_till;
use winnow::combinator::separated;
use winnow::combinator::separated_pair;
use winnow::combinator::terminated;
use winnow::combinator::trace;
use winnow::error::AddContext;
use winnow::error::ErrMode;
use winnow::error::ErrorKind;
use winnow::error::ParserError;
use winnow::error::StrContext;
use winnow::error::StrContextValue;
use winnow::stream::AsChar;
use winnow::stream::Stateful;
use winnow::stream::Stream;
use winnow::token::take_till;
use winnow::token::take_while;
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

pub type AsmPResult<I> = winnow::PResult<I, PError>;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum PError {
    #[error("unknown dialect '{0}'")]
    UnknownDialect(String),
    #[error("unknown operation '{1}' in dialect '{0}'")]
    UnknownOperation(String, String),
    #[error("expected '{0}'")]
    ExpectedNotFound(String),
    #[error("unknown type '{0}'")]
    UnknownType(String),
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

impl<I: Stream> AddContext<I, StrContext> for PError {
    fn add_context(
        self,
        _input: &I,
        _token_start: &<I as Stream>::Checkpoint,
        context: StrContext,
    ) -> Self {
        match context {
            StrContext::Expected(value) => match value {
                StrContextValue::CharLiteral(value) => PError::ExpectedNotFound(value.to_string()),
                StrContextValue::StringLiteral(value) => {
                    PError::ExpectedNotFound(value.to_string())
                }
                StrContextValue::Description(value) => PError::ExpectedNotFound(value.to_string()),
                _ => todo!(),
            },
            _ => PError::Unknown,
        }
    }
}

pub trait Parsable<T> {
    fn parse(input: &mut ParseStream<'_>) -> AsmPResult<T>;
}

pub fn expected_token(token: &'static str, input: &mut ParseStream<'_>) -> AsmPResult<()> {
    delimited(multispace0, token, multispace0)
        .context(StrContext::Expected(StrContextValue::StringLiteral(token)))
        .void()
        .parse_next(input)
}

pub fn identifier<'s>(input: &mut ParseStream<'s>) -> AsmPResult<&'s str> {
    let ident_sign = take_while(0.., |c: char| c.is_alphanum() || c.as_char() == '_');
    (alpha1, ident_sign).take().parse_next(input)
}

fn dialect_op<'s>(input: &mut ParseStream<'s>) -> AsmPResult<(&'s str, &'s str)> {
    separated_pair(identifier, ".", identifier).parse_next(input)
}

fn builtin_op<'s>(input: &mut ParseStream<'s>) -> AsmPResult<(&'s str, &'s str)> {
    identifier
        .take()
        .parse_next(input)
        .map(|op| ("builtin", op))
}

pub fn op_tuple<'s>(input: &mut ParseStream<'s>) -> AsmPResult<(&'s str, &'s str)> {
    alt((dialect_op, builtin_op)).parse_next(input)
}

pub fn sym_name<'s>(input: &mut ParseStream<'s>) -> AsmPResult<&'s str> {
    preceded("@", identifier).parse_next(input)
}

pub fn word<'a, F, O, E: ParserError<ParseStream<'a>>>(
    inner: F,
) -> impl Parser<ParseStream<'a>, O, E>
where
    F: Parser<ParseStream<'a>, O, E>,
{
    delimited(space0, inner, space0)
}

pub fn skip_attrs(
    _input: &mut ParseStream<'_>,
) -> AsmPResult<std::collections::HashMap<String, Attr>> {
    let res: std::collections::HashMap<String, Attr> = HashMap::new();
    Ok(res)
}

fn single_comment(input: &mut ParseStream<'_>) -> AsmPResult<()> {
    (';', take_till(1.., ['\n', '\r']), line_ending)
        .void()
        .parse_next(input)
}

fn comment(input: &mut ParseStream<'_>) -> AsmPResult<()> {
    repeat(0.., preceded(multispace0, single_comment)).parse_next(input)
}

pub fn single_op(input: &mut ParseStream) -> AsmPResult<OpRef> {
    let context = input.state.get_context();

    let skip = trace(
        "skip comments",
        alt((terminated(comment, multispace0), multispace0.map(|_| ()))),
    );

    let (dialect_name, op_name) = trace("op name", preceded(skip, op_tuple)).parse_next(input)?;

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

pub fn single_block_region(ir: &mut ParseStream<'_>) -> AsmPResult<Vec<OpRef>> {
    expected_token("{", ir)?;

    let operations = repeat(0.., single_op).parse_next(ir)?;

    expected_token("}", ir)?;

    Ok(operations)
}

pub fn single_block(input: &mut ParseStream<'_>) -> AsmPResult<BlockRef> {
    let skip = trace(
        "skip comments",
        alt((terminated(comment, multispace0), multispace0.map(|_| ()))),
    );
    let block_name = preceded(
        (skip, '^'),
        terminated(
            identifier,
            (
                cut_err(':'.context(StrContext::Expected(StrContextValue::CharLiteral(':')))),
                multispace0,
            ),
        ),
    )
    .parse_next(input)?;

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

pub fn region_with_blocks(input: &mut ParseStream<'_>) -> AsmPResult<RegionRef> {
    expected_token("{", input)?;
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

fn attr_pair(input: &mut ParseStream<'_>) -> AsmPResult<(String, Attr)> {
    trace(
        "attr pair",
        separated_pair(
            identifier.map(|s| s.to_string()),
            (space0, "=", space0),
            Attr::parse,
        ),
    )
    .parse_next(input)
}

pub fn attr_list(input: &mut ParseStream<'_>) -> AsmPResult<HashMap<String, Attr>> {
    let attr_pairs = separated::<_, _, HashMap<_, _>, _, _, _, _>(
        0..,
        attr_pair,
        (space0, ",", space0),
    );
    trace(
        "attribute list",
        terminated(
            preceded((space0, "attrs", space0, "=", space0, "{"), attr_pairs),
            (space0, "}", space0),
        ),
    )
    .parse_next(input)
}

fn parse_digits<'s>(input: &mut ParseStream<'s>) -> AsmPResult<&'s str> {
    winnow::token::take_while(1.., '0'..='9').parse_next(input)
}

pub fn parse_int_bits<'s>(input: &mut ParseStream<'s>) -> AsmPResult<HashMap<String, Attr>> {
    let parse_int_bits_impl = |input: &mut ParseStream<'s>| {
        terminated(preceded("<", parse_digits), ">").parse_next(input)
    };
    let bits_str = parse_int_bits_impl(input)?;
    let maybe_bits_num: AsmPResult<u32> = match str::parse(bits_str) {
        Ok(n) => Ok(n),
        Err(..) => Err(winnow::error::ErrMode::Cut(PError::ExpectedNotFound(
            String::from("integer"),
        ))),
    };
    let bits_num = maybe_bits_num?;
    let mut r = HashMap::<String, Attr>::new();
    r.insert("bits".into(), Attr::U32(bits_num));
    Ok(r)
}

pub fn print_parser_diag(
    context: ContextRef,
    diag: &winnow::error::ParseError<ParseStream<'_>, PError>,
) {
    let offset = diag.offset();
    let inner = diag.inner();

    let mut builder = Report::<std::ops::Range<usize>>::build(ReportKind::Error, (), offset)
        .with_config(
            Config::default()
                .with_tab_width(2)
                .with_index_type(ariadne::IndexType::Byte),
        )
        .with_message(&format!("{}", &inner));

    match inner {
        PError::UnknownOperation(dialect_name, op_name) => {
            builder.add_label(
                Label::new((offset - op_name.len())..offset)
                    .with_color(Color::Red)
                    .with_message("unknown operation"),
            );
            if let Some(dialect) = context.get_dialect_by_name(dialect_name) {
                if let Some(name) = dialect.get_similarly_named_op(op_name) {
                    builder.set_note(format!(
                        "there is a similarly named operation '{}' in '{}' dialect",
                        name, &dialect_name
                    ));
                }
            }
        }
        PError::ExpectedNotFound(token) => {
            builder.add_label(
                Label::new(offset..(offset + token.len()))
                    .with_message("unexpected token")
                    .with_color(Color::Red),
            );
        }
        _ => {
            builder.add_label(Label::new(offset..(offset + 1)).with_message("unexpected token"));
            #[cfg(debug_assertions)]
            builder.set_note("for development purpose try re-building with --features=winnow/debug")
        }
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
