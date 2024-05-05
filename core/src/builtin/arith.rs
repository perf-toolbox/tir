use crate::builtin::DIALECT_NAME;
use crate::{Op, OpImpl, Type};
use tir_macros::Op;

use crate as tir_core;

#[derive(Op)]
#[operation(name = "const", known_attrs(value: IntegerAttr))]
pub struct ConstOp {
    #[ret_type]
    return_type: Type,
    r#impl: OpImpl,
}

#[cfg(test)]
mod test {
    use std::any::TypeId;

    use crate::builtin::*;
    use crate::Attr;
    use crate::Context;
    use crate::OpBuilder;

    use super::*;

    #[test]
    fn test_const_op() {
        assert!(ConstOp::get_operation_name() == "const");

        let context = Context::new();
        let module = ModuleOp::builder(context.clone()).build();
        let builder = OpBuilder::new(context.clone(), module.borrow().get_body());

        let attr = Attr::I8(16);

        let ret_type = VoidType::build(context.clone());
        let constant = ConstOp::builder(context.clone())
            .value(attr)
            .return_type(ret_type.into())
            .build();

        builder.insert(&constant);
        assert_eq!(
            TryInto::<i8>::try_into(constant.borrow().get_value_attr().clone()).unwrap(),
            16
        );
        let body = module.borrow().get_body().clone();
        let op = body.first().unwrap();
        assert_eq!((*op.borrow()).type_id(), TypeId::of::<ConstOp>());
    }
}
