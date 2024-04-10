use std::cell::RefCell;
use std::rc::Rc;

use crate::{Attr, Context, Op, Operation, Type};
use crate::builtin::DIALECT_NAME;

pub struct FuncOp {
    operation: Rc<RefCell<Operation>>,
}

impl FuncOp {
    pub fn new(
        context: Rc<RefCell<Context>>,
        name: String,
        input_types: &[Type],
        return_type: Type,
    ) -> Self {
        let dialect = context.borrow().get_dialect_by_name(DIALECT_NAME).unwrap();
        let operation = Operation::new(context.clone(), dialect, FuncOp::get_operation_name());

        let region = operation.borrow_mut().emplace_region();

        region.borrow_mut().emplace_block(Rc::downgrade(&region));

        operation
            .borrow_mut()
            .add_attr("sym_name".to_string(), Attr::String(name));

        Self { operation }
    }

    pub fn sym_name(&self) -> String {
        match self.operation.borrow().get_attrs().get("sym_name").unwrap() {
            Attr::String(name) => name.clone(),
            _ => panic!("sym_name is not a string"),
        }
    }
}

impl Op for FuncOp {
    fn get_operation_name() -> &'static str {
        "func"
    }
}

impl Into<Rc<RefCell<Operation>>> for FuncOp {
    fn into(self) -> Rc<RefCell<Operation>> {
        self.operation.clone()
    }
}

impl TryFrom<Rc<RefCell<Operation>>> for FuncOp {
    type Error = ();

    fn try_from(operation: Rc<RefCell<Operation>>) -> Result<Self, Self::Error> {
        if operation.borrow().get_operation_name() != FuncOp::get_operation_name()
            || operation.borrow().get_dialect_id()
                != operation
                    .borrow()
                    .get_context()
                    .borrow()
                    .get_dialect_by_name(DIALECT_NAME)
                    .unwrap()
                    .borrow()
                    .get_id()
        {
            return Err(());
        }

        Ok(Self { operation })
    }
}

#[cfg(test)]
mod test {
    use crate::{Context, Op};
    use crate::builtin::*;

    use super::*;

    #[test]
    fn test_module() {
        assert!(FuncOp::get_operation_name() == "func");

        let context = Context::new();
        let module = ModuleOp::new(context.clone());

        let inputs = vec![];
        let result = VoidType::build(context.clone());

        let func = func::FuncOp::new(context, "test".to_string(), &inputs, result.into());
        module.get_body().borrow_mut().add_operation(func.into());

        match FuncOp::try_from(
            module
                .get_body()
                .borrow()
                .get_operations()
                .first()
                .unwrap()
                .clone(),
        ) {
            Ok(func) => {
                assert!(func.sym_name() == "test");
            }
            Err(_) => panic!("convert to func failed"),
        }
    }
}
