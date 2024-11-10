use crate::target::SectionOp;
use crate::{AsmToken, TokenStream};
use lpl::combinators::one_of;
use lpl::{Diagnostic, ParseStream, Parser};
use tir_core::{Block, Region};

pub fn parse_asm<'a>(
    input: TokenStream<'a>,
    instr_parsers: &'a [Box<dyn Parser<'a, TokenStream<'a>, ()>>],
) -> Result<((), Option<TokenStream<'a>>), Diagnostic> {
    let parser = section().or_else(label()).or_else(one_of(instr_parsers));

    parser.parse(input)
}

fn directive_as_str<'a>() -> impl Parser<'a, TokenStream<'a>, &'a str> {
    move |stream: TokenStream<'a>| match stream.get(0..1).unwrap()[0].0 {
        AsmToken::Directive(d) => Ok((d, stream.slice(1..stream.len()))),
        _ => todo!(), /*Err(ParserError::new(
                          "expected a directive".to_string(),
                          stream.span(),
                      ))*/,
    }
}

fn known_section<'a>() -> impl Parser<'a, TokenStream<'a>, &'a str> {
    directive_as_str().try_map(|d, s| match d {
        "text" => Ok(d),
        "data" => Ok(d),
        _ => todo!(), /*Err(ParserError::new(
                          "expected a name of a known directive".to_string(),
                          s,
                      ))*/,
    })
}

pub fn asm_ident<'a>() -> impl Parser<'a, TokenStream<'a>, &'a str> {
    move |input: TokenStream<'a>| {
        let first = input.peek().unwrap();
        match first {
            AsmToken::Ident(ident) => Ok((ident, input.slice(1..input.len().clone()))),
            _ => todo!(), /*Err(ParserError::new("expected ident".to_string(), input.span()))*/,
        }
    }
}

fn asm_label<'a>() -> impl Parser<'a, TokenStream<'a>, &'a str> {
    move |input: TokenStream<'a>| {
        let first = input.peek().unwrap();
        match first {
            AsmToken::Label(ident) => Ok((ident, input.slice(1..input.len().clone()))),
            _ => todo!(), /*Err(ParserError::new("expected label".to_string(), input.span()))*/,
        }
    }
}

fn generic_section<'a>() -> impl Parser<'a, TokenStream<'a>, &'a str> {
    directive_as_str()
        .try_map(|d, s| match d {
            "section" => Ok(()),
            _ => todo!(), /*Err(ParserError::new("expected 'section'".to_string(), s))*/,
        })
        .and_then(asm_ident())
        .map(|(_, name)| name)
}

fn section<'a>() -> impl Parser<'a, TokenStream<'a>, ()> {
    known_section()
        .or_else(generic_section())
        .map_with(|name, asm_ctx| {
            let asm_ctx = asm_ctx.unwrap();
            let builder = asm_ctx.get_builder();
            let context = builder.get_context();

            let section = if let Some(section) = asm_ctx.get_section(name) {
                section
            } else {
                let body = Region::empty(&context);
                let section = SectionOp::builder(&context)
                    .name(name.to_string().into())
                    .body(body)
                    .build();
                builder.insert(&section);
                asm_ctx.add_section(name, &section);
                section
            };

            asm_ctx.set_active_section(section);
        })
}

pub fn label<'a>() -> impl Parser<'a, TokenStream<'a>, ()> {
    asm_label().map_with(|name, asm_ctx| {
        let asm_ctx = asm_ctx.unwrap();
        let builder = asm_ctx.get_builder();

        let section = asm_ctx.get_active_section().unwrap();
        let parent = section.borrow().get_body_region();
        let block = Block::with_arguments::<&str>(name, &parent, &[], &[]);

        parent.add_block(block.clone());

        builder.set_insertion_point_to_start(block);
    })
}

#[cfg(test)]
mod tests {
    use crate::{lex_asm, target::create_dialect, TokenStream};

    use super::section;
    use lpl::Parser;
    use tir_core::{builtin::ModuleOp, Context, OpBuilder};

    #[test]
    fn section_parser() {
        let context = Context::new();

        let module = ModuleOp::builder(&context).build();
        let builder = OpBuilder::new(context.clone(), module.borrow().get_body());
        context.add_dialect(create_dialect());

        let input = ".section text";

        // label:";
        let tokens = lex_asm(input).expect("lex");
        println!("{:?}", &tokens);
        let stream = TokenStream::new(&tokens, builder);

        section().parse(stream).expect("section");
    }
}
