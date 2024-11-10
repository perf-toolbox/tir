use std::collections::HashMap;

use ariadne::Color;
use ariadne::Config;
use ariadne::Label;
use ariadne::Report;
use ariadne::ReportKind;
use lpl::combinators::any_whitespace1;
use lpl::combinators::interleaved;
use lpl::combinators::lang::ident;
use lpl::combinators::literal;
use lpl::combinators::separated_ignore;
use lpl::combinators::spaced;
use lpl::combinators::zero_or_more;
use lpl::Diagnostic;
use lpl::ParseResult;
use lpl::{ParseStream, Parser};
use thiserror::Error;

use crate::assembly::ir_stream::IRStrStream;
use crate::Attr;
use crate::Block;
use crate::BlockRef;
use crate::OpBuilder;
use crate::Region;
use crate::RegionRef;
use crate::{ContextRef, OpRef, Type};

use super::DiagKind;

/// Basic trait that any operation has to implement
pub trait Parsable<T> {
    fn parse(input: IRStrStream) -> ParseResult<IRStrStream, T>;
}

/// Parse textual TIR into inner structures
pub fn parse_ir(context: ContextRef, input: &str) -> Result<OpRef, Diagnostic> {
    let stream = IRStrStream::new(input, context);

    let parser = single_op();

    let (op, _) = parser.parse(stream)?;
    Ok(op)
}

/// Parse TIR @-style symbol names
pub fn sym_name<'a>() -> impl Parser<'a, IRStrStream<'a>, &'a str> {
    literal("@")
        .and_then(identifier())
        .map(|(_, sym)| sym)
        .label("sym_name")
}

/// Parse generic TIR identifier
pub fn identifier<'a>() -> impl Parser<'a, IRStrStream<'a>, &'a str> {
    ident(|c| c == '_' || c == '.').label("identifier")
}

/// Parse all operations inside a single basic block region.
/// Syntax is:
/// ```tir
/// {
///     op1
///     op2
/// }
/// ```
pub fn single_block_region<'a>() -> impl Parser<'a, IRStrStream<'a>, Vec<OpRef>> {
    spaced(literal("{"))
        .and_then(zero_or_more(single_op()))
        .and_then(spaced(literal("}")))
        .flat()
        .map(|(_, ops, _)| ops)
        .label("single_block_region")
}

/// Parse attributes list.
///
/// Syntax example:
/// ```tir
/// attrs = {attr1 = <str: "Hello, World!">, attr2 = <i8: 42>}
/// ```
pub fn attr_list<'a>() -> impl Parser<'a, IRStrStream<'a>, HashMap<String, Attr>> {
    let single_attribute = identifier()
        .and_then(spaced(literal("=")))
        .and_then(Attr::parse)
        .map(|((name, _), attr)| (name.to_string(), attr));

    let attr_pairs = separated_ignore(single_attribute, spaced(literal(",")).void());

    literal("attrs")
        .and_then(spaced(literal("=")))
        .and_then(spaced(literal("{")))
        .and_then(attr_pairs)
        .and_then(spaced(literal("}")))
        .flat()
        .try_map(|(_, _, _, pairs, _), span| {
            let mut map = HashMap::new();
            for (k, v) in pairs.iter() {
                if map.contains_key(k) {
                    return Err(DiagKind::DuplicateAttr(k.clone(), span).into());
                }

                map.insert(k.clone(), v.clone());
            }
            Ok(map)
        })
        .label("attr_list")
}

pub fn skip_attrs<'a>() -> impl Parser<'a, IRStrStream<'a>, HashMap<String, Attr>> {
    any_whitespace1().map(|_| HashMap::new())
}

/// Generic operation name
fn op_name<'a>() -> impl Parser<'a, IRStrStream<'a>, (&'a str, &'a str)> {
    dialect_op().or_else(builtin_op()).label("op_name")
}

/// dialect_name.op_name -> (dialect_name, op_name)
fn dialect_op<'a>() -> impl Parser<'a, IRStrStream<'a>, (&'a str, &'a str)> {
    ident(|c| c == '_')
        .and_then(literal("."))
        .and_then(identifier())
        .map(|((d, _), o)| (d, o))
        .label("dialect_op")
}

/// "builtin op"-style identifier
fn builtin_op<'a>() -> impl Parser<'a, IRStrStream<'a>, (&'a str, &'a str)> {
    ident(|c| c == '_')
        .map(|o| ("builtin", o))
        .label("builtin_op")
}

fn single_op<'a>() -> impl Parser<'a, IRStrStream<'a>, OpRef> {
    let parser = move |input: IRStrStream<'a>| {
        let ((dialect_name, op_name), next_input) = spaced(op_name()).parse(input.clone())?;

        // It is impossible to construct IRStrStream without a context
        let context = input.get_extra().unwrap();

        let dialect = context
            .get_dialect_by_name(dialect_name)
            .ok_or(Into::<Diagnostic>::into(DiagKind::UnknownDialect(
                dialect_name.to_owned(),
                input.span(),
            )))?;

        let operation_id = dialect
            .get_operation_id(op_name)
            .ok_or(Into::<Diagnostic>::into(DiagKind::UnknownOperation(
                op_name.to_owned(),
                dialect_name.to_owned(),
                input.span(),
            )))?;

        // It is impossible to add an operation without specifying its parser
        let parser = dialect.get_operation_parser(operation_id).unwrap();
        parser.parse(next_input.unwrap())
    };

    parser.label("single_op")
}

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

#[cfg(test)]
mod tests {
    use super::{attr_list, Attr};
    use crate::IRStrStream;
    use lpl::Parser;

    #[test]
    fn test_attr_list() {
        let context = crate::Context::new(); // Assuming Context is defined in crate
        let input = "attrs = {attr1 = <str: \"Hello, World!\">, attr2 = <i8: 42>}";
        let input = IRStrStream::new(input, context);
        let result = attr_list().parse(input);
        assert!(result.is_ok());
        let (attrs, _) = result.unwrap();
        println!("{:?}", attrs);
        assert_eq!(
            attrs.get("attr1").unwrap(),
            &Attr::String("Hello, World!".to_string())
        );
        assert_eq!(attrs.get("attr2").unwrap(), &Attr::I8(42));
    }

    #[test]
    fn test_attr_list_duplicate() {
        let context = crate::Context::new();
        let input = "attrs = {attr1 = <str: \"Hello\">, attr1 = <str: \"World\">}";
        let input = IRStrStream::new(input, context);
        let result = attr_list().parse(input);
        assert!(result.is_err());
    }
}
