use crate::builtin::DIALECT_NAME;
use crate::parser::{single_block_region, AsmPResult, ParseStream};
use crate::{IRFormatter, Op, OpAssembly, OpImpl, OpRef, Printable, RegionRef, Terminator};
use tir_macros::{op_implements, Op, OpAssembly, OpValidator};
use winnow::Parser;

use crate as tir_core;

#[derive(Op, Debug, OpValidator)]
#[operation(name = "module")]
pub struct ModuleOp {
    #[region(single_block, no_args)]
    body: RegionRef,
    r#impl: OpImpl,
}

#[derive(Op, Debug, OpValidator, OpAssembly)]
#[operation(name = "module_end")]
pub struct ModuleEndOp {
    r#impl: OpImpl,
}

#[op_implements]
impl Terminator for ModuleEndOp {}

impl OpAssembly for ModuleOp {
    fn parse_assembly(input: &mut ParseStream) -> AsmPResult<OpRef>
    where
        Self: Sized,
    {
        let ops = single_block_region.parse_next(input)?;
        let context = input.state.get_context();
        let module = ModuleOp::builder(&context).build();
        for op in ops {
            module.borrow_mut().get_body().push(&op);
        }
        Ok(module)
    }

    fn print_assembly(&self, fmt: &mut dyn IRFormatter) {
        fmt.start_region();
        let body = self.get_body();
        for op in body.iter() {
            op.borrow().print(fmt);
        }
        fmt.end_region();
    }
}

#[cfg(test)]
mod test {
    use std::any::TypeId;

    use super::*;
    use crate::{parse_ir, Context, Printable, StringPrinter};

    #[test]
    fn test_module() {
        assert!(ModuleOp::get_operation_name() == "module");

        let context = Context::new();
        let module = ModuleOp::builder(&context).build();
        module.borrow().get_body_region();
        module.borrow().get_body();
        module.borrow().get_context();
    }

    // TODO replace this test with a snapshot test
    #[test]
    fn test_module_print() {
        let context = Context::new();
        let module = ModuleOp::builder(&context).build();

        let mut printer = StringPrinter::new();

        module.borrow().print(&mut printer);

        let result = printer.get();

        let golden = "module {\n}\n";
        assert_eq!(result, golden);
    }

    #[test]
    fn test_module_parse() {
        let context = Context::new();
        let input = "module {\n}\n";
        let op = parse_ir(context, input).expect("parsed ir");
        assert_eq!(op.borrow().type_id(), TypeId::of::<ModuleOp>());
    }
}
