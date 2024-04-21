use std::cell::RefCell;
use std::rc::Rc;

use crate::builtin::FuncType;
use crate::builtin::DIALECT_NAME;
use crate::utils::{trait_id, TraitId};
use crate::*;
use tir_macros::operation;

#[operation(name = "func")]
pub struct FuncOp {
    #[cfg(region = true)]
    body: Type,
    #[cfg(attribute = true)]
    sym_name: String,
    #[cfg(attribute = true)]
    func_type: FuncType,
}

#[cfg(test)]
mod test {
    use std::any::TypeId;

    use crate::{builtin::*, OpBuilder};
    use crate::{Context, Op};

    use super::*;

    #[test]
    fn test_module() {
        assert!(FuncOp::get_operation_name() == "func");

        let context = Context::new();
        let module = ModuleOp::builder(context.clone()).build();
        let builder = OpBuilder::new(context.clone(), module.borrow().get_body());

        let inputs: Vec<Type> = vec![];
        let result = VoidType::build(context.clone());

        let func_type = FuncType::build(context.clone(), &inputs, result.into());

        let func = func::FuncOp::builder(context)
            .sym_name("test".to_string().into())
            .func_type(func_type.into())
            .build();
        builder.borrow_mut().insert(func.clone());
        assert_eq!(
            TryInto::<String>::try_into(func.borrow().get_sym_name_attr()).unwrap(),
            "test"
        );
        let body = module.borrow().get_body().clone();
        let op = body.borrow().get_operations().first().unwrap().clone();
        assert_eq!((*op.borrow()).type_id(), TypeId::of::<FuncOp>());
    }
}
