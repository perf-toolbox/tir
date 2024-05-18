use tir_core::parser::AsmPResult;
use tir_core::{Block, OpBuilder, Region};
use winnow::combinator::{peek};
use winnow::error::StrContext;
use winnow::Parser;
use winnow::{
    token::{one_of},
    Stateful,
};

use crate::target::SectionOp;
use crate::{AsmToken, TokenStream};

// use crate::target::SectionOp;

#[derive(Debug, Clone)]
pub struct AsmParserState {
    builder: OpBuilder,
}

impl AsmParserState {
    pub fn new(builder: OpBuilder) -> Self {
        Self { builder }
    }
    pub fn get_builder(&self) -> OpBuilder {
        self.builder.clone()
    }
}

pub type AsmStream<'a> = Stateful<&'a str, AsmParserState>;

pub fn section(input: &mut TokenStream<'_, '_>) -> AsmPResult<()> {
    let s: AsmToken = one_of(|t| matches!(t, AsmToken::Section(_))).parse_next(input)?;
    peek(one_of(|t| matches!(t, AsmToken::Label(_))))
        .context(StrContext::Expected(
            "Expected label name after section".into(),
        ))
        .parse_next(input)?;

    let AsmToken::Section(name) = s else {
        unreachable!()
    };
    let builder = input.get_builder();
    let context = builder.get_context();

    let section = if let Some(section) = input.get_section(name) {
        section
    } else {
        let body = Region::empty(&context);
        SectionOp::builder(&context)
            .name(name.to_string().into())
            .body(body)
            .build()
    };

    input.set_active_section(section);

    Ok(())
}

pub fn label(input: &mut TokenStream<'_, '_>) -> AsmPResult<()> {
    let l = one_of(|t| matches!(t, AsmToken::Label(_))).parse_next(input)?;

    let AsmToken::Label(name) = l else {
        unreachable!()
    };

    let section = input.get_active_section().unwrap();
    let parent = section.borrow().get_body_region();

    let block = Block::with_arguments::<&str>(name, &parent, &[], &[]);

    parent.add_block(block.clone());

    let builder = input.get_builder();
    builder.set_insertion_point_to_start(block);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{lex_asm, target::create_dialect, TokenStream};

    use super::section;
    use tir_core::{builtin::ModuleOp, Context, OpBuilder};
    use winnow::Parser;

    #[test]
    fn section_parser() {
        let context = Context::new();

        let module = ModuleOp::builder(&context).build();
        let builder = OpBuilder::new(context.clone(), module.borrow().get_body());
        context.add_dialect(create_dialect());

        let input = ".section .text


label:";
        let tokens = lex_asm(input).expect("lex");
        let mut stream = TokenStream::new(&builder, &tokens);

        section.parse_next(&mut stream).expect("section");
    }
}
