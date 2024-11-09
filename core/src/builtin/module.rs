use crate::builtin::DIALECT_NAME;
use crate::parser::single_block_region;
// use crate::parser::single_block_region;
use crate::{
    IRFormatter, IRStrStream, Op, OpAssembly, OpImpl, OpRef, Printable, RegionRef, Terminator,
};
use lpl::{ParseResult, ParseStream, Parser};
use tir_macros::{op_implements, Op, OpAssembly, OpValidator};

use crate as tir_core;

#[derive(Op, Debug, OpValidator)]
#[operation(name = "module", dialect = builtin)]
pub struct ModuleOp {
    #[region(single_block, no_args)]
    body: RegionRef,
    r#impl: OpImpl,
}

#[derive(Op, Debug, OpValidator, OpAssembly)]
#[operation(name = "module_end", dialect = builtin)]
pub struct ModuleEndOp {
    r#impl: OpImpl,
}

#[op_implements(dialect = builtin)]
impl Terminator for ModuleEndOp {}

impl OpAssembly for ModuleOp {
    fn parse_assembly<'a>(input: IRStrStream<'_>) -> ParseResult<IRStrStream<'_>, OpRef>
    where
        Self: Sized,
    {
        let parser = single_block_region();
        let context = input.get_extra().unwrap().clone();
        parser.parse(input).map(|(ops, next_input)| {
            let module = ModuleOp::builder(&context).build();
            for op in ops {
                module.borrow_mut().get_body().push(&op);
            }

            let module: OpRef = module;

            (module, next_input)
        })
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
    use crate::{
        parse_ir,
        utils::{op_dyn_cast, op_has_trait},
        Context, Printable, StringPrinter,
    };

    #[test]
    fn test_module() {
        assert!(ModuleOp::get_operation_name() == "module");

        let context = Context::new();
        let module = ModuleOp::builder(&context).build();
        module.borrow().get_body_region();
        module.borrow().get_body();
        module.borrow().get_context();
        assert!(!op_has_trait::<dyn Terminator>(module.clone()));
        assert!(op_dyn_cast::<dyn Terminator>(module).is_none());
    }

    #[test]
    fn test_module_end() {
        assert!(ModuleEndOp::get_operation_name() == "module_end");

        let context = Context::new();

        let module_end = ModuleEndOp::builder(&context).build();
        assert!(op_has_trait::<dyn Terminator>(module_end.clone()));
        assert!(op_dyn_cast::<dyn Terminator>(module_end).is_some());
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
