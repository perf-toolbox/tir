use winnow::ascii::alpha1;
use winnow::ascii::alphanumeric0;
use winnow::combinator::alt;
use winnow::combinator::separated_pair;
use winnow::PResult;
use winnow::Parser;

use crate::{ContextRef, Operation};

fn identifier<'s>(input: &mut &'s str) -> PResult<&'s str> {
    (alpha1, alphanumeric0).recognize().parse_next(input)
}

fn dialect_op<'s>(input: &mut &'s str) -> PResult<(&'s str, &'s str)> {
    separated_pair(identifier, ".", identifier).parse_next(input)
}

fn builtin_op<'s>(input: &mut &'s str) -> PResult<(&'s str, &'s str)> {
    identifier
        .recognize()
        .parse_next(input)
        .map(|op| ("builtin", op))
}

fn op_tuple<'s>(input: &mut &'s str) -> PResult<(&'s str, &'s str)> {
    alt((dialect_op, builtin_op)).parse_next(input)
}

pub fn parse_op(context: ContextRef, ir: &str) -> Result<Operation, ()> {
    let mut input = ir;
    let (dialect_name, op_name) = op_tuple.parse_next(&mut input).map_err(|_| ())?;

    let dialect = context
        .borrow()
        .get_dialect_by_name(dialect_name)
        .ok_or(())?;

    let operation_id = dialect.borrow().get_operation_id(op_name).ok_or(())?;
    let parser = dialect
        .borrow()
        .get_operation_parser(operation_id)
        .ok_or(())?;
    parser(context, &mut input)
}

#[cfg(test)]
mod tests {
    use winnow::Parser;

    use super::{identifier, op_tuple};

    #[test]
    fn parse_ident() {
        assert!(identifier.parse("abc").is_ok());
        assert!(identifier.parse("abc123").is_ok());
        assert!(identifier.parse("123").is_err());
        assert!(identifier.parse("123abc").is_err());
        let mut input = "abc123 abc 123";
        let ident = identifier.parse_next(&mut input).unwrap();
        assert_eq!(ident, "abc123");
    }

    #[test]
    fn parse_op_name() {
        let mut ir = "module";
        let result = op_tuple.parse_next(&mut ir).unwrap();
        assert_eq!(result, ("builtin", "module"));

        let mut ir = "test.module";
        let result = op_tuple.parse_next(&mut ir).unwrap();
        assert_eq!(result, ("test", "module"));
    }
}
