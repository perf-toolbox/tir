use std::{cell::RefCell, rc::Rc};

use crate::builtin::DIALECT_NAME;
use crate::utils::{trait_id, TraitId};
use crate::*;
use tir_macros::operation;

#[operation(name = "module", custom_assembly = true)]
pub struct ModuleOp {
    #[cfg(region = true, single_block = true)]
    body: Region,
}

impl IRAssembly for ModuleOp {
    fn parse(_: ContextRef, _input: &mut &str) -> std::result::Result<Operation, ()>
    where
        Self: Sized,
    {
        todo!();
    }

    fn print(&self, fmt: &mut dyn IRFormatter) {
        fmt.start_region();
        let body = self.get_body();
        for op in &body.borrow().operations {
            op.borrow().print(fmt);
        }
        fmt.end_region();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_module() {
        assert!(ModuleOp::get_operation_name() == "module");

        let context = Context::new();
        let module = ModuleOp::builder(context).build();
        module.borrow().get_body_region();
        module.borrow().get_body();
        module.borrow().get_region();
    }

    // TODO replace this test with a snapshot test
    #[test]
    fn test_module_print() {
        let context = Context::new();
        let module = ModuleOp::builder(context).build();

        let mut printer = StdoutPrinter::new();

        print_op(module, &mut printer);
        // panic!();
    }
}
