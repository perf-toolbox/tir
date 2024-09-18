use std::collections::HashMap;

use ariadne::Color;
use ariadne::Config;
use ariadne::Label;
use ariadne::Report;
use ariadne::ReportKind;
use lpl::combinators::literal;
use lpl::combinators::text::ident;
use lpl::{ParseStream, Parser, ParserError};
use thiserror::Error;

use crate::assembly::ir_stream::IRStrStream;
use crate::Attr;
use crate::Block;
use crate::BlockRef;
use crate::OpBuilder;
use crate::Region;
use crate::RegionRef;
use crate::{ContextRef, OpRef, Type};

/// Basic trait that any operation has to implement
pub trait Parsable<T> {
    fn parse<'a>() -> Box<dyn Parser<'a, IRStrStream<'a>, OpRef>>;
}

/// Parse textual TIR into inner structures
pub fn parse_ir(context: ContextRef, input: &str) -> Result<OpRef, ParserError> {
    let stream = IRStrStream::new(input, context);
    // let stream: StrS
    todo!()
}

/// Parse TIR @-style symbol names
pub fn sym_name<'a>() -> impl Parser<'a, IRStrStream<'a>, &'a str> {
    literal("@").and_then(ident(valid_op_char)).map(|(_, sym)| sym)
}

/// Parse generic TIR identifier
pub fn identifier<'a>() -> impl Parser<'a, IRStrStream<'a>, &'a str> {
    ident(|c| c == '_' || c == '.')
}

/// Generic operation name
fn op_name<'a>() -> impl Parser<'a, IRStrStream<'a>, (&'a str, &'a str)> {
   dialect_op().or_else(builtin_op())
}

/// dialect_name.op_name -> (dialect_name, op_name)
fn dialect_op<'a>() -> impl Parser<'a, IRStrStream<'a>, (&'a str, &'a str)> {
    ident(|c| c == '_').and_then(literal(".")).and_then(identifier()).map(|((d, _), o)| (d, o))
}

/// "builtin op"-style identifier
fn builtin_op<'a>() -> impl Parser<'a, IRStrStream<'a>, (&'a str, &'a str)> {
    ident(|c| c == '_').map(|o| ("builtin", o))
}

fn single_op<'a>() -> impl Parser<'a, IRStrStream<'a>, OpRef> {
    move |input: IRStrStream| {
        let ((dialect_name, op_name), next_input) = op_name().parse(input.clone())?;

        // It is impossible to construct IRStrStream without a context
        let context = input.get_extra().unwrap();

        let dialect = context
            .get_dialect_by_name(dialect_name)
            .ok_or(ParserError::new(
                format!("unknown dialect '{}'", dialect_name),
                input.span(),
            ))?;

        let operation_id = dialect.get_operation_id(op_name).ok_or(ParserError::new(
            format!(
                "unknown operation '{}' in dialect '{}'",
                op_name, dialect_name
            ),
            input.span(),
        ))?;

        // It is impossible to add an operation without specifying its parser
        let parser = dialect.get_operation_parser(operation_id).unwrap();
        parser.parse(next_input.unwrap())
    }
}

// pub fn word<'a, F, O, E: ParserError<ParseStream<'a>>>(
//     inner: F,
// ) -> impl Parser<ParseStream<'a>, O, E>
// where
//     F: Parser<ParseStream<'a>, O, E>,
// {
//     delimited(space0, inner, space0)
// }
//
// pub fn skip_attrs(
//     _input: &mut ParseStream<'_>,
// ) -> AsmPResult<std::collections::HashMap<String, Attr>> {
//     let res: std::collections::HashMap<String, Attr> = HashMap::new();
//     Ok(res)
// }
//
// fn single_comment(input: &mut ParseStream<'_>) -> AsmPResult<()> {
//     (';', take_till(1.., ['\n', '\r']), line_ending)
//         .void()
//         .parse_next(input)
// }
//
// fn comment(input: &mut ParseStream<'_>) -> AsmPResult<()> {
//     repeat(0.., preceded(multispace0, single_comment)).parse_next(input)
// }
// pub fn single_block_region(ir: &mut ParseStream<'_>) -> AsmPResult<Vec<OpRef>> {
//     expected_token("{", ir)?;
//
//     let operations = repeat(0.., single_op).parse_next(ir)?;
//
//     expected_token("}", ir)?;
//
//     Ok(operations)
// }
//
// pub fn single_block(input: &mut ParseStream<'_>) -> AsmPResult<BlockRef> {
//     let skip = trace(
//         "skip comments",
//         alt((terminated(comment, multispace0), multispace0.map(|_| ()))),
//     );
//     let block_name = preceded(
//         (skip, '^'),
//         terminated(
//             identifier,
//             (
//                 cut_err(':'.context(StrContext::Expected(StrContextValue::CharLiteral(':')))),
//                 multispace0,
//             ),
//         ),
//     )
//     .parse_next(input)?;
//
//     let ops: Vec<OpRef> = repeat(0.., single_op).parse_next(input)?;
//
//     let region = input.state.get_region();
//
//     let names = input.state.take_deferred_names();
//     let types = input.state.take_deferred_types();
//     let block = Block::with_arguments(block_name, &region, &types, &names);
//
//     for op in ops {
//         block.push(&op);
//     }
//
//     Ok(block)
// }
//
// pub fn region_with_blocks(input: &mut ParseStream<'_>) -> AsmPResult<RegionRef> {
//     expected_token("{", input)?;
//     let context = input.state.get_context();
//     let region = Region::empty(&context);
//     input.state.push_region(region.clone());
//
//     let (blocks, (_, _)): (Vec<BlockRef>, (_, _)) =
//         repeat_till(1.., single_block, (multispace0, "}")).parse_next(input)?;
//
//     for block in blocks {
//         region.add_block(block);
//     }
//
//     input.state.pop_region();
//
//     Ok(region)
// }
//
// fn attr_pair(input: &mut ParseStream<'_>) -> AsmPResult<(String, Attr)> {
//     trace(
//         "attr pair",
//         separated_pair(
//             identifier.map(|s| s.to_string()),
//             (space0, "=", space0),
//             Attr::parse,
//         ),
//     )
//     .parse_next(input)
// }
//
// pub fn attr_list(input: &mut ParseStream<'_>) -> AsmPResult<HashMap<String, Attr>> {
//     let attr_pairs =
//         separated::<_, _, HashMap<_, _>, _, _, _, _>(0.., attr_pair, (space0, ",", space0));
//     trace(
//         "attribute list",
//         terminated(
//             preceded((space0, "attrs", space0, "=", space0, "{"), attr_pairs),
//             (space0, "}", space0),
//         ),
//     )
//     .parse_next(input)
// }
//
// fn parse_digits<'s>(input: &mut ParseStream<'s>) -> AsmPResult<&'s str> {
//     winnow::token::take_while(1.., '0'..='9').parse_next(input)
// }
//
// pub fn parse_int_bits<'s>(input: &mut ParseStream<'s>) -> AsmPResult<HashMap<String, Attr>> {
//     let parse_int_bits_impl = |input: &mut ParseStream<'s>| {
//         terminated(preceded("<", parse_digits), ">").parse_next(input)
//     };
//     let bits_str = parse_int_bits_impl(input)?;
//     let maybe_bits_num: AsmPResult<u32> = match str::parse(bits_str) {
//         Ok(n) => Ok(n),
//         Err(..) => Err(winnow::error::ErrMode::Cut(PError::ExpectedNotFound(
//             String::from("integer"),
//         ))),
//     };
//     let bits_num = maybe_bits_num?;
//     let mut r = HashMap::<String, Attr>::new();
//     r.insert("bits".into(), Attr::U32(bits_num));
//     Ok(r)
// }
//
// pub fn print_parser_diag(
//     context: ContextRef,
//     diag: &winnow::error::ParseError<ParseStream<'_>, PError>,
// ) {
//     let offset = diag.offset();
//     let inner = diag.inner();
//
//     let mut builder = Report::<std::ops::Range<usize>>::build(ReportKind::Error, (), offset)
//         .with_config(
//             Config::default()
//                 .with_tab_width(2)
//                 .with_index_type(ariadne::IndexType::Byte),
//         )
//         .with_message(&format!("{}", &inner));
//
//     match inner {
//         PError::UnknownOperation(dialect_name, op_name) => {
//             builder.add_label(
//                 Label::new((offset - op_name.len())..offset)
//                     .with_color(Color::Red)
//                     .with_message("unknown operation"),
//             );
//             if let Some(dialect) = context.get_dialect_by_name(dialect_name) {
//                 if let Some(name) = dialect.get_similarly_named_op(op_name) {
//                     builder.set_note(format!(
//                         "there is a similarly named operation '{}' in '{}' dialect",
//                         name, &dialect_name
//                     ));
//                 }
//             }
//         }
//         PError::ExpectedNotFound(token) => {
//             builder.add_label(
//                 Label::new(offset..(offset + token.len()))
//                     .with_message("unexpected token")
//                     .with_color(Color::Red),
//             );
//         }
//         _ => {
//             builder.add_label(Label::new(offset..(offset + 1)).with_message("unexpected token"));
//             #[cfg(debug_assertions)]
//             builder.set_note("for development purpose try re-building with --features=winnow/debug")
//         }
//     }
//
//     builder
//         .finish()
//         .print(ariadne::Source::from(diag.input().input))
//         .unwrap();
// }
//
// #[cfg(test)]
// mod tests {
//     use winnow::Parser;
//
//     use super::{identifier, op_tuple, ParseStream, ParserState};
//     use crate::Context;
//
//     macro_rules! input {
//         ($inp:literal, $context:expr) => {
//             ParseStream {
//                 input: $inp.into(),
//                 state: ParserState::new($context),
//             }
//         };
//     }
//
//     #[test]
//     fn parse_ident() {
//         let context = Context::new();
//         assert!(identifier.parse(input!("abc", context.clone())).is_ok());
//         assert!(identifier.parse(input!("abc123", context.clone())).is_ok());
//         assert!(identifier.parse(input!("123", context.clone())).is_err());
//         assert!(identifier.parse(input!("123abs", context.clone())).is_err());
//         let mut inp = input!("abc123 abc 123", context.clone());
//         let ident = identifier.parse_next(&mut inp).unwrap();
//         assert_eq!(ident, "abc123");
//     }
//
//     #[test]
//     fn parse_op_name() {
//         let context = Context::new();
//         let mut ir = input!("module", context.clone());
//         let result = op_tuple.parse_next(&mut ir).unwrap();
//         assert_eq!(result, ("builtin", "module"));
//
//         let mut ir = input!("test.module", context.clone());
//         let result = op_tuple.parse_next(&mut ir).unwrap();
//         assert_eq!(result, ("test", "module"));
//     }
// }
