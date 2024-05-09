use winnow::error::ErrorKind;
use winnow::stream::Stream;

pub type PResult<I> = winnow::prelude::PResult<I, ParserError<I>>;

#[derive(Debug, PartialEq, Eq)]
pub enum ParserError<I> {
    Nom(I, ErrorKind),
}

impl<I: Stream + Clone> winnow::error::ParserError<I> for ParserError<I> {
    fn from_error_kind(input: &I, kind: ErrorKind) -> Self {
        ParserError::Nom(input.clone(), kind)
    }

    fn append(self, _: &I, _: &<I as Stream>::Checkpoint, _: ErrorKind) -> Self {
        self
    }
}

// use winnow::ascii::alpha1;
// use winnow::ascii::alphanumeric0;
// use winnow::ascii::multispace0;
// use winnow::combinator::alt;
// use winnow::combinator::separated_pair;
// use winnow::error::ContextError;
// use winnow::error::ErrMode;
// use winnow::error::StrContext;
// // use winnow::PResult;
// use winnow::Parser;
// use winnow::error::ErrorKind;
// use winnow::stream::Stream;

// use crate::{ContextRef, OpRef};

// #[derive(Debug, PartialEq, Eq)]
// pub enum ParserError<I> {
//     Nom(I, ErrorKind),
// }

// impl<I: Stream + Clone> winnow::error::ParserError<I> for ParserError<I> {
//     fn from_error_kind(input: &I, kind: ErrorKind) -> Self {
//         ParserError::Nom(input.clone(), kind)
//     }

//     fn append(self, _: &I, _: &<I as Stream>::Checkpoint, _: ErrorKind) -> Self {
//         self
//     }
// }

// type PResult<T, I = ContextError> = Result<T, ErrMode<ParserError<I>>>;

// fn identifier<'s>(input: &mut &'s str) -> PResult<&'s str> {
//     (alpha1, alphanumeric0).recognize().parse_next(input)
// }

// fn dialect_op<'s>(input: &mut &'s str) -> PResult<(&'s str, &'s str)> {
//     separated_pair(identifier, ".", identifier).parse_next(input)
// }

// fn builtin_op<'s>(input: &mut &'s str) -> PResult<(&'s str, &'s str)> {
//     identifier
//         .recognize()
//         .parse_next(input)
//         .map(|op| ("builtin", op))
// }

// fn op_tuple<'s>(input: &mut &'s str) -> PResult<(&'s str, &'s str)> {
//     alt((dialect_op, builtin_op)).parse_next(input)
// }

// pub fn parse_ir(context: ContextRef, ir: &str) -> Result<OpRef, ()> {
//     let mut input = ir;
//     let (dialect_name, op_name) = op_tuple.parse_next(&mut input).map_err(|_| ())?;

//     let dialect = context.get_dialect_by_name(dialect_name).ok_or(())?;

//     let operation_id = dialect.get_operation_id(op_name).ok_or(())?;
//     let parser = dialect.get_operation_parser(operation_id).ok_or(())?;
//     parser(context, &mut input)
// }

// pub fn parse_single_operation(context: &ContextRef, ir: &mut &str) -> PResult<OpRef> {
//     let ir = ir;
//     let (dialect_name, op_name) = op_tuple
//         .parse_next(ir)
//         .map_err(|_| ErrMode::Backtrack(ContextError::new()))?;

//     let dialect = context
//         .get_dialect_by_name(dialect_name)
//         .ok_or(ErrMode::Backtrack(ContextError::new()))?;

//     let operation_id = dialect
//         .get_operation_id(op_name)
//         .ok_or(ErrMode::Backtrack(ContextError::new()))?;
//     let parser = dialect
//         .get_operation_parser(operation_id)
//         .ok_or(ErrMode::Backtrack(ContextError::new()))?;
//     parser(context.clone(), ir).map_err(|_| ErrMode::Backtrack(ContextError::new()))
// }

// pub fn parse_single_block_region(context: ContextRef, ir: &mut &str) -> Result<Vec<OpRef>, ()> {
//     let _ = (multispace0, "{", multispace0)
//         .parse_next(ir)
//         .map_err(|_: ErrMode<ContextError>| ())?;

//     let mut operations = vec![];

//     loop {
//         if let Ok(operation) = parse_single_operation(&context, ir) {
//             operations.push(operation);
//         } else {
//             break;
//         }
//     }

//     let _ = (multispace0, "}")
//         .parse_next(ir)
//         .map_err(|_: ErrMode<ContextError>| ())?;

//     Ok(operations)
// }

// #[cfg(test)]
// mod tests {
//     use winnow::Parser;

//     use super::{identifier, op_tuple};

//     #[test]
//     fn parse_ident() {
//         assert!(identifier.parse("abc").is_ok());
//         assert!(identifier.parse("abc123").is_ok());
//         assert!(identifier.parse("123").is_err());
//         assert!(identifier.parse("123abc").is_err());
//         let mut input = "abc123 abc 123";
//         let ident = identifier.parse_next(&mut input).unwrap();
//         assert_eq!(ident, "abc123");
//     }

//     #[test]
//     fn parse_op_name() {
//         let mut ir = "module";
//         let result = op_tuple.parse_next(&mut ir).unwrap();
//         assert_eq!(result, ("builtin", "module"));

//         let mut ir = "test.module";
//         let result = op_tuple.parse_next(&mut ir).unwrap();
//         assert_eq!(result, ("test", "module"));
//     }
// }
