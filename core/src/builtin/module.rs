use std::{cell::RefCell, rc::Rc};

use crate::builtin::DIALECT_NAME;
use crate::utils::{trait_id, TraitId};
use crate::{Block, Context, Op, Operation, OperationImpl, Region};
use tir_macros::operation;

#[operation(name = "module")]
pub struct ModuleOp {
    #[cfg(region = true, single_block = true)]
    body: Region,
}

impl ModuleOp {
    pub fn new(context: Rc<RefCell<Context>>) -> Self {
        let dialect = context.borrow().get_dialect_by_name(DIALECT_NAME).unwrap();
        let mut operation =
            Operation::new(context.clone(), dialect, ModuleOp::get_operation_name());

        let region = operation.emplace_region();

        region.borrow_mut().emplace_block(Rc::downgrade(&region));

        Self {
            operation: operation.get_impl().clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_module() {
        assert!(ModuleOp::get_operation_name() == "module");

        let context = Context::new();
        let module = ModuleOp::new(context);
        module.get_body_region();
        module.get_body();
        module.get_region();
    }
}
