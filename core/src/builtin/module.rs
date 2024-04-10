use std::{cell::RefCell, rc::Rc};

use crate::{Block, Context, Op, Operation, Region};
use crate::builtin::DIALECT_NAME;

#[derive(Debug)]
pub struct ModuleOp {
    operation: Rc<RefCell<Operation>>,
}

impl ModuleOp {
    pub fn new(context: Rc<RefCell<Context>>) -> Self {
        let dialect = context.borrow().get_dialect_by_name(DIALECT_NAME).unwrap();
        let operation = Operation::new(context.clone(), dialect, ModuleOp::get_operation_name());

        let region = operation.borrow_mut().emplace_region();

        region.borrow_mut().emplace_block(Rc::downgrade(&region));

        Self { operation }
    }

    pub fn get_region(&self) -> Rc<RefCell<Region>> {
        self.operation
            .borrow()
            .get_regions()
            .first()
            .unwrap()
            .clone()
    }

    pub fn get_body(&self) -> Rc<RefCell<Block>> {
        self.operation
            .borrow()
            .get_regions()
            .first()
            .unwrap()
            .borrow()
            .get_blocks()
            .first()
            .unwrap()
            .clone()
    }
}

impl Into<Rc<RefCell<Operation>>> for ModuleOp {
    fn into(self) -> Rc<RefCell<Operation>> {
        self.operation.clone()
    }
}

impl Op for ModuleOp {
    fn get_operation_name() -> &'static str {
        "module"
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
        module.get_region();
    }
}
