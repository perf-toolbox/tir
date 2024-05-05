use crate::builtin::DIALECT_NAME;
use crate::{
    parse_single_block_region, Assembly, ContextRef, IRFormatter, Op, OpImpl, OpRef, RegionRef,
};
use tir_macros::Op;

use crate as tir_core;

#[derive(Op, Debug)]
#[operation(name = "module")]
pub struct ModuleOp {
    #[region(single_block, no_args)]
    body: RegionRef,
    r#impl: OpImpl,
}

impl Assembly for ModuleOp {
    fn parse(context: ContextRef, input: &mut &str) -> std::result::Result<OpRef, ()>
    where
        Self: Sized,
    {
        let ops = parse_single_block_region(context.clone(), input)?;
        let module = ModuleOp::builder(&context).build();
        for op in ops {
            module.borrow_mut().get_body().push(&op);
        }
        Ok(module)
    }

    fn print(&self, fmt: &mut dyn IRFormatter) {
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
    use crate::{parse_ir, print_op, Context, StringPrinter};

    #[test]
    fn test_module() {
        assert!(ModuleOp::get_operation_name() == "module");

        let context = Context::new();
        let module = ModuleOp::builder(&context).build();
        module.borrow().get_body_region();
        module.borrow().get_body();
        module.borrow().get_context();
        eprint!("{:?}", context);
        // module.borrow().get_region();
    }

    // // TODO replace this test with a snapshot test
    #[test]
    fn test_module_print() {
        let context = Context::new();
        let module = ModuleOp::builder(&context).build();
    
        let mut printer = StringPrinter::new();
    
        print_op(module, &mut printer);
    
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
