use std::cell::RefCell;
use std::rc::Rc;

use tir_backend::parser::{AsmParserState, AsmStream};
use tir_core::parser::PError;
use tir_core::{builtin::ModuleOp, ContextRef, OpBuilder};
use winnow::ascii::multispace0;
use winnow::combinator::{preceded, repeat, terminated};
use winnow::Parser;

use crate::r_instr;

pub fn parse_asm<'a>(
    context: &ContextRef,
    input: &'a str,
) -> Result<Rc<RefCell<ModuleOp>>, winnow::error::ParseError<AsmStream<'a>, PError>> {
    let module = ModuleOp::builder(context).build();
    let builder = OpBuilder::new(context.clone(), module.borrow().get_body());
    let stream = AsmStream {
        input,
        state: AsmParserState::new(builder.clone()),
    };

    repeat(
        0..,
        preceded(
            /*comment, */ multispace0,
            terminated(r_instr, /*comment, */ multispace0),
        ),
    )
    .parse(stream)?;

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

        assert!(parse_asm(&context, input).is_ok());
    }
}
