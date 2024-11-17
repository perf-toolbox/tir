use std::collections::HashMap;
use std::sync::Arc;

use ariadne::Config;
use ariadne::Label;
use ariadne::Report;
use ariadne::ReportKind;
use lpl::combinators::any_whitespace0;
use lpl::combinators::any_whitespace1;
use lpl::combinators::lang::ident;
use lpl::combinators::lang::line_comment;
use lpl::combinators::literal;
use lpl::combinators::maybe_then;
use lpl::combinators::one_or_more;
use lpl::combinators::optional;
use lpl::combinators::separated_ignore;
use lpl::combinators::spaced;
use lpl::combinators::text::take_while;
use lpl::combinators::zero_or_more;
use lpl::Diagnostic;
use lpl::ParseResult;
use lpl::{ParseStream, Parser};

use crate::assembly::ir_stream::{IRStrStream, ParserState};
use crate::Attr;
use crate::Block;
use crate::BlockRef;
use crate::Region;
use crate::RegionRef;
use crate::{ContextRef, OpRef};

use super::DiagKind;

/// Basic trait that any operation has to implement
pub trait Parsable<T> {
    fn parse(input: IRStrStream) -> ParseResult<IRStrStream, T>;
}

/// Parse textual TIR into inner structures
pub fn parse_ir(context: ContextRef, input: &str, filename: &str) -> Result<OpRef, Diagnostic> {
    let stream = IRStrStream::new(input, filename, context);

    let parser = single_op();

    let (op, _) = parser.parse(stream)?;
    Ok(op)
}

pub fn print_parser_diag(source: &str, diag: &Diagnostic) {
    let span = diag.span();
    let offset = span.get_offset_start();

    let builder = Report::build(
        ReportKind::Error,
        (
            span.get_filename().unwrap_or_default(),
            offset..(span.get_offset_end()),
        ),
    )
    .with_config(
        Config::default()
            .with_tab_width(2)
            .with_index_type(ariadne::IndexType::Byte),
    )
    .with_message("Failed to parse IR")
    .with_label(
        Label::new((
            span.get_filename().unwrap_or_default(),
            offset..(offset + 1),
        ))
        .with_message(diag.message()),
    );

    println!("{:?}", diag);

    builder
        .finish()
        .print((
            span.get_filename().unwrap_or_default(),
            ariadne::Source::from(source),
        ))
        .unwrap();
    // FIXME(alexbatashev): restore fuzzy dialect and op name search
    // match inner {
    //     PError::UnknownOperation(dialect_name, op_name) => {
    //         builder.add_label(
    //             Label::new((offset - op_name.len())..offset)
    //                 .with_color(Color::Red)
    //                 .with_message("unknown operation"),
    //         );
    //         if let Some(dialect) = context.get_dialect_by_name(dialect_name) {
    //             if let Some(name) = dialect.get_similarly_named_op(op_name) {
    //                 builder.set_note(format!(
    //                     "there is a similarly named operation '{}' in '{}' dialect",
    //                     name, &dialect_name
    //                 ));
    //             }
    //         }
    //     }
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
    any_whitespace0().map(|_| HashMap::new())
}

pub fn single_block<'a>() -> impl Parser<'a, IRStrStream<'a>, BlockRef> {
    let skip = zero_or_more(any_whitespace1().or_else(line_comment(";").void()));
    skip.and_then(literal("^"))
        .and_then(ident(|_| false))
        .and_then(literal(":"))
        .flat()
        .map(|(_, _, name, _)| name)
        .and_then(one_or_more(single_op()))
        .map_with(|(block_name, ops), extra| {
            let state = extra.unwrap();

            let region = state.get_region();

            let names = state.take_deferred_names();
            let types = state.take_deferred_types();
            let block = Block::with_arguments(block_name, &region, &types, &names);

            for op in ops {
                block.push(&op);
            }

            block
        })
}

pub fn region_with_blocks<'a>() -> impl Parser<'a, IRStrStream<'a>, RegionRef> {
    spaced(literal("{"))
        .map_with(|_, extra| {
            let state: &Arc<ParserState> = extra.unwrap();
            let context = state.context();

            let region = Region::empty(&context);
            state.push_region(region.clone());

            region
        })
        .and_then(one_or_more(single_block()))
        .and_then(any_whitespace0().and_then(literal("}")))
        .map_with(|((region, blocks), _), extra| {
            let state: &Arc<ParserState> = extra.unwrap();

            for block in blocks {
                region.add_block(block);
            }

            state.pop_region();

            region
        })
}

pub fn parse_int_bits<'a>() -> impl Parser<'a, IRStrStream<'a>, HashMap<String, Attr>> {
    literal("<")
        .and_then(take_while(|c| c.is_numeric()))
        .and_then(literal(">"))
        .flat()
        .map(|(_, bits, _)| {
            let bits = bits.parse::<u32>().unwrap();
            let mut attrs = HashMap::new();
            attrs.insert("bits".into(), Attr::U32(bits));

            attrs
        })
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
        let state = input.get_extra().unwrap();
        let context = state.context();

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

    maybe_then(
        optional(
            zero_or_more(
                any_whitespace1()
                    .and_then(line_comment(";"))
                    .and_then(any_whitespace0()),
            )
            .label("eat comments"),
        )
        .and_then(parser.label("single_op")),
        zero_or_more(line_comment(";")),
    )
    .map(|((_, op), _)| op)
}

#[cfg(test)]
mod tests {
    use super::{attr_list, Attr};
    use crate::{parser::op_name, IRStrStream};
    use lpl::Parser;

    #[test]
    fn test_attr_list() {
        let context = crate::Context::new();
        let input = "attrs = {attr1 = <str: \"Hello, World!\">, attr2 = <i8: 42>}";
        let input = IRStrStream::new(input, "-", context);
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
        let input = IRStrStream::new(input, "-", context);
        let result = attr_list().parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn parse_op_name() {
        let context = crate::Context::new();
        let input = "module";
        let input = IRStrStream::new(input, "-", context.clone());

        let (result, _) = op_name().parse(input).unwrap();
        assert_eq!(result, ("builtin", "module"));

        let input = "test.module";
        let input = IRStrStream::new(input, "-", context);
        let (result, _) = op_name().parse(input).unwrap();
        assert_eq!(result, ("test", "module"));
    }
}
