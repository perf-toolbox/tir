use std::cell::RefCell;
use std::rc::Rc;

use tir_backend::parser::{label, section};
use tir_backend::{lex_asm, TokenStream};
use tir_core::parser::{AsmPResult, PError};
use tir_core::{builtin::ModuleOp, ContextRef, OpBuilder};
use winnow::combinator::{alt, repeat};
use winnow::Parser;

use crate::RVExt;

fn asm_instr(input: &mut TokenStream<'_, '_>) -> AsmPResult<()> {
    let builder = input.get_builder();
    let context = builder.get_context();
    let dialect = context.get_dialect_by_name(crate::DIALECT_NAME).unwrap();

    let mut parsers = dialect
        .get_dialect_extension()
        .unwrap()
        .downcast_ref::<RVExt>()
        .unwrap()
        .get_asm_parsers();

    for p in &mut parsers {
        if p.parse_next(input).is_ok() {
            return Ok(());
        }
    }

    Err(winnow::error::ErrMode::Backtrack(PError::Unknown))
}

pub fn parse_asm<'a>(
    context: &ContextRef,
    input: &'a str,
) -> Result<Rc<RefCell<ModuleOp>>, winnow::error::ParseError<TokenStream<'a, 'a>, PError>> {
    let module = ModuleOp::builder(context).build();
    let builder = OpBuilder::new(context.clone(), module.borrow().get_body());

    let tokens = lex_asm(input);
    if let Err(ref err) = tokens {
        panic!("lexer failed: {}", err);
    }
    let tokens = tokens.unwrap();
    println!("Tokens: {:?}\n", &tokens);
    let stream = TokenStream::new(&builder, &tokens);

    let _: Vec<()> = repeat(0.., alt((section, label, asm_instr)))
        .parse(stream)
        .expect("todo err handling");

    Ok(module)
}

#[cfg(test)]
mod tests {
    use tir_core::Context;

    use crate::parse_asm;

    #[test]
    fn simple_parse() {
        let input = "add x28, x6, x7
sub x28, x6, x7
sll x28, x6, x7
slt x28, x6, x7
sltu x28, x6, x7
srl x28, x6, x7
sra x28, x6, x7
or x28, x6, x7
and x28, x6, x7";

        let context = Context::new();
        context.add_dialect(crate::create_dialect());
        context.add_dialect(tir_backend::target::create_dialect());

        assert!(parse_asm(&context, input).is_ok());
    }
}
