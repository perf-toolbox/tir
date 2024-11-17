use crate::target::SectionOp;
use crate::{AsmToken, DiagKind, TokenStream};
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

pub fn asm_ident<'a>() -> impl Parser<'a, TokenStream<'a>, &'a str> {
    let parser = move |input: TokenStream<'a>| {
        let first = input.peek().unwrap();
        match first {
            AsmToken::Ident(ident) => Ok((ident, input.slice(1..input.len()))),
            _ => Err(Into::<Diagnostic>::into(DiagKind::ExpectedIdent(
                input.span(),
            ))),
        }
    };

    parser.label("asm_ident")
}

pub fn comma<'a>() -> impl Parser<'a, TokenStream<'a>, ()> {
    let parser = move |input: TokenStream<'a>| {
        if input.len() == 0 {
            return Err(Into::<Diagnostic>::into(DiagKind::EndOfStream));
        }
        let first = input.peek().unwrap();
        match first {
            AsmToken::Comma => Ok(((), input.slice(1..input.len()))),
            _ => Err(Into::<Diagnostic>::into(DiagKind::UnexpectedToken(
                input.span(),
            ))),
        }
    };

    parser.label("asm_comma")
}

pub fn open_paren<'a>() -> impl Parser<'a, TokenStream<'a>, ()> {
    let parser = move |input: TokenStream<'a>| {
        if input.len() == 0 {
            return Err(Into::<Diagnostic>::into(DiagKind::EndOfStream));
        }
        let first = input.peek().unwrap();
        match first {
            AsmToken::OpenParen => Ok(((), input.slice(1..input.len()))),
            _ => Err(Into::<Diagnostic>::into(DiagKind::UnexpectedToken(
                input.span(),
            ))),
        }
    };

    parser.label("open_paren")
}

pub fn close_paren<'a>() -> impl Parser<'a, TokenStream<'a>, ()> {
    let parser = move |input: TokenStream<'a>| {
        if input.len() == 0 {
            return Err(Into::<Diagnostic>::into(DiagKind::EndOfStream));
        }
        let first = input.peek().unwrap();
        match first {
            AsmToken::CloseParen => Ok(((), input.slice(1..input.len()))),
            _ => Err(Into::<Diagnostic>::into(DiagKind::UnexpectedToken(
                input.span(),
            ))),
        }
    };

    parser.label("close_paren")
}

pub fn number<'a>() -> impl Parser<'a, TokenStream<'a>, i64> {
    let parser = move |input: TokenStream<'a>| {
        if input.len() == 0 {
            return Err(Into::<Diagnostic>::into(DiagKind::EndOfStream));
        }
        let first = input.peek().unwrap();
        match first {
            AsmToken::Number(num) => Ok((num, input.slice(1..input.len()))),
            _ => Err(Into::<Diagnostic>::into(DiagKind::UnexpectedToken(
                input.span(),
            ))),
        }
    };

    parser.label("number")
}

pub fn section<'a>() -> impl Parser<'a, TokenStream<'a>, ()> {
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
                // FIXME(alexbatashev): need a better way to handle section names
                let section = SectionOp::builder(&context)
                    .name(format!(".{}", name).into())
                    .body(body)
                    .build();
                builder.insert(&section);
                asm_ctx.add_section(name, &section);
                section
            };

            asm_ctx.set_active_section(section);
        })
        .label("asm_section")
}

pub fn label<'a>() -> impl Parser<'a, TokenStream<'a>, ()> {
    asm_label()
        .map_with(|name, asm_ctx| {
            let asm_ctx = asm_ctx.unwrap();
            let builder = asm_ctx.get_builder();

            let section = asm_ctx.get_active_section().unwrap();
            let parent = section.borrow().get_body_region();
            let block = Block::with_arguments::<&str>(name, &parent, &[], &[]);

            parent.add_block(block.clone());

            builder.set_insertion_point_to_start(block);
        })
        .label("asm_label")
}

fn asm_label<'a>() -> impl Parser<'a, TokenStream<'a>, &'a str> {
    move |input: TokenStream<'a>| {
        let first = input.peek().unwrap();
        match first {
            AsmToken::Label(ident) => Ok((ident, input.slice(1..input.len()))),
            _ => Err(Into::<Diagnostic>::into(DiagKind::ExpectedLabel(
                input.span(),
            ))),
        }
    }
}

fn generic_section<'a>() -> impl Parser<'a, TokenStream<'a>, &'a str> {
    directive_as_str()
        .try_map(|d, s| match d {
            "section" => Ok(()),
            _ => Err(Into::<Diagnostic>::into(
                DiagKind::ExpectedSpecificDirective("section", s),
            )),
        })
        .and_then(asm_ident())
        .map(|(_, name)| name)
        .label("generic_section")
}

fn directive_as_str<'a>() -> impl Parser<'a, TokenStream<'a>, &'a str> {
    move |stream: TokenStream<'a>| match stream.get(0..1).unwrap()[0].0 {
        AsmToken::Directive(d) => Ok((d, stream.slice(1..stream.len()))),
        _ => Err(Into::<Diagnostic>::into(DiagKind::ExpectedDirective(
            stream.span(),
        ))),
    }
}

fn known_section<'a>() -> impl Parser<'a, TokenStream<'a>, &'a str> {
    directive_as_str()
        .try_map(|d, s| match d {
            "text" => Ok(d),
            "data" => Ok(d),
            _ => Err(Into::<Diagnostic>::into(DiagKind::ExpectedDirective(s))),
        })
        .label("known_section")
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

        let input = ".section text

label:";
        let tokens = lex_asm(input).expect("lex");
        println!("{:?}", &tokens);
        let stream = TokenStream::new(&tokens, builder);

        section().parse(stream).expect("section");
    }
}
