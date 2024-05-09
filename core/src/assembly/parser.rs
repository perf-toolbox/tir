use winnow::ascii::alpha1;
use winnow::ascii::alphanumeric0;
use winnow::ascii::multispace0;
use winnow::combinator::alt;
use winnow::combinator::separated_pair;
use winnow::error::ContextError;
use winnow::error::ErrMode;
use winnow::stream::Stateful;
use winnow::Parser;

use crate::{ContextRef, OpRef};

#[derive(Debug, Clone)]
pub struct ParserState {
    context: ContextRef,
}

impl ParserState {
    pub fn get_context(&self) -> ContextRef {
        self.context.clone()
    }
}

pub type ParseStream<'a> = Stateful<&'a str, ParserState>;

pub type PResult<I> = winnow::PResult<I>;

fn identifier<'s>(input: &mut ParseStream<'s>) -> PResult<&'s str> {
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

fn op_tuple<'s>(input: &mut ParseStream<'s>) -> PResult<(&'s str, &'s str)> {
    alt((dialect_op, builtin_op)).parse_next(input)
}

pub fn single_op(input: &mut ParseStream) -> PResult<OpRef> {
    let context = input.state.get_context();
    let (dialect_name, op_name) = op_tuple.parse_next(input)?;

    let dialect = context
        .get_dialect_by_name(dialect_name)
        .ok_or(ErrMode::Backtrack(ContextError::new()))?;

    let operation_id = dialect
        .get_operation_id(op_name)
        .ok_or(ErrMode::Backtrack(ContextError::new()))?;
    let mut parser = dialect
        .get_operation_parser(operation_id)
        .ok_or(ErrMode::Backtrack(ContextError::new()))?;
    parser.parse_next(input)
}

pub fn parse_ir(
    context: ContextRef,
    input: &str,
) -> Result<OpRef, winnow::error::ParseError<ParseStream<'_>, winnow::error::ContextError>> {
    let input = ParseStream {
        input,
        state: ParserState { context },
    };

    single_op.parse(input)
}

pub fn single_block_region(ir: &mut ParseStream<'_>) -> PResult<Vec<OpRef>> {
    let _ = (multispace0, "{", multispace0).parse_next(ir)?;

    let mut operations = vec![];

    while let Ok(operation) = single_op.parse_next(ir) {
        operations.push(operation);
    }

    let _ = (multispace0, "}", multispace0).parse_next(ir)?;

    Ok(operations)
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
                state: ParserState { context: $context },
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
