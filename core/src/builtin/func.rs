use std::cell::RefCell;
use std::rc::Rc;

use crate::builtin::DIALECT_NAME;
use crate::utils::{trait_id, TraitId};
use crate::{Attr, Context, Op, Operation, OperationImpl, Region, Type};
use tir_macros::operation;

#[operation(name = "func")]
pub struct FuncOp {
    #[cfg(region = true)]
    body: Type,
}

impl FuncOp {
    pub fn new(
        context: Rc<RefCell<Context>>,
        name: String,
        input_types: &[Type],
        return_type: Type,
    ) -> Self {
        let dialect = context.borrow().get_dialect_by_name(DIALECT_NAME).unwrap();
        let mut operation = Operation::new(context.clone(), dialect, FuncOp::get_operation_name());

        let region = operation.emplace_region();

        region.borrow_mut().emplace_block(Rc::downgrade(&region));

        operation.add_attr("sym_name".to_string(), Attr::String(name));

        Self {
            operation: operation.get_impl(),
        }
    }

    pub fn sym_name(&self) -> String {
        match self.operation.borrow().attrs.get("sym_name").unwrap() {
            Attr::String(name) => name.clone(),
            _ => panic!("sym_name is not a string"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::builtin::*;
    use crate::{Context, Op};

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
