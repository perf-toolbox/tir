use std::cell::RefCell;
use std::rc::Rc;

use lpl::combinators::lang::line_comment;
use lpl::combinators::{eof, zero_or_more};
use lpl::{Diagnostic, ParseResult, ParseStream, Parser};
use tir_backend::parser::{label, section};
use tir_backend::{lex_asm, TokenStream};
use tir_core::{builtin::ModuleOp, ContextRef, OpBuilder};

use crate::{DiagKind, RVExt};

fn asm_instr(input: TokenStream) -> ParseResult<TokenStream, ()> {
    let asm_ctx = input.get_extra().unwrap();
    let builder = asm_ctx.get_builder();
    let context = builder.get_context();
    let dialect = context.get_dialect_by_name(crate::DIALECT_NAME).unwrap();

    let parsers = dialect
        .get_dialect_extension()
        .unwrap()
        .downcast_ref::<RVExt>()
        .unwrap()
        .get_asm_parsers();

    for p in parsers {
        let result = p.parse(input.clone());
        if result.is_ok() {
            return result;
        }
    }

    Err(DiagKind::UnknownOpcode(input.span()).into())
}

#[allow(clippy::result_large_err)]
pub fn parse_asm(context: &ContextRef, input: &str) -> Result<Rc<RefCell<ModuleOp>>, Diagnostic> {
    let module = ModuleOp::builder(context).build();
    let builder = OpBuilder::new(context.clone(), module.borrow().get_body());

    let tokens = lex_asm(input)?;

    let stream = TokenStream::new(&tokens, builder);

    let parser = eof(zero_or_more(
        section()
            .or_else(label())
            .or_else(asm_instr.label("asm_instr")),
    ));

    parser.parse(stream)?;
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
