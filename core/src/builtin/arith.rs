use std::cell::RefCell;
use std::rc::Rc;

use crate::builtin::DIALECT_NAME;
use crate::utils::{trait_id, TraitId};
use crate::Type;
use crate::*;
use tir_macros::operation;

#[operation(name = "const", return_type = Type)]
pub struct ConstOp {
    #[cfg(attribute = true)]
    value: Attr,
}

#[cfg(test)]
mod test {
    use std::any::TypeId;

    use crate::{builtin::*, OpBuilder};
    use crate::{Context, Op};

    use super::*;

    #[test]
    fn test_const_op() {
        assert!(ConstOp::get_operation_name() == "const");

        let context = Context::new();
        let module = ModuleOp::builder(context.clone()).build();
        let builder = OpBuilder::new(context.clone(), module.borrow().get_body());

        let value_attr = Attr::I8(16);
        let value_type = IntegerType::build(context.clone(), true, 8);

        let constant = ConstOp::builder(context.clone())
            .value(value_attr.into())
            .return_type(value_type.into())
            .build();

        builder.borrow_mut().insert(constant.clone());
        assert_eq!(
            TryInto::<i8>::try_into(constant.borrow().get_value_attr()).unwrap(),
            16
        );
        // FIXME: add test
        assert_eq!(constant.borrow().get_return_type(), value_type);
        let body = module.borrow().get_body().clone();
        let op = body.borrow().get_operations().first().unwrap().clone();
        assert_eq!((*op.borrow()).type_id(), TypeId::of::<ConstOp>());
    }
}
