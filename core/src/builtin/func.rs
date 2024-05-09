use crate::builtin::DIALECT_NAME;
use crate::*;
use tir_macros::Op;
use tir_macros::OpAssembly;
use winnow::Parser;

use crate as tir_core;

#[derive(Op, OpAssembly)]
#[operation(name = "func", known_attrs(sym_name: String, func_type: Type))]
pub struct FuncOp {
    #[region]
    body: RegionRef,
    r#impl: OpImpl,
}

#[cfg(test)]
mod test {
    use std::any::TypeId;

    use crate::Context;
    use crate::{builtin::*, OpBuilder};

    use super::*;

    #[test]
    fn test_module() {
        assert!(FuncOp::get_operation_name() == "func");

        let context = Context::new();
        let module = ModuleOp::builder(&context).build();
        let builder = OpBuilder::new(context.clone(), module.borrow().get_body());

        let inputs: Vec<Type> = vec![];
        let result = VoidType::build(context.clone());

        let func_type = FuncType::build(context.clone(), &inputs, result.into());

        let func = func::FuncOp::builder(&context)
            .sym_name("test".to_string().into())
            .func_type(func_type.into())
            .body(Region::empty(&context))
            .build();
        builder.insert(&func);
        assert_eq!(
            TryInto::<String>::try_into(func.borrow().get_sym_name_attr().clone()).unwrap(),
            "test"
        );
        let body = module.borrow().get_body().clone();
        let op = body.first().unwrap().clone();
        assert_eq!((*op.borrow()).type_id(), TypeId::of::<FuncOp>());
    }
}
