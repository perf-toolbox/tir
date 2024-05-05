use crate::builtin::DIALECT_NAME;
use crate::{Op, OpImpl, Type};
use tir_macros::Op;

use crate as tir_core;

#[derive(Op)]
#[operation(name = "const", known_attrs(value: IntegerAttr))]
pub struct ConstOp {
    #[ret_type]
    rtype: Type,
    r#impl: OpImpl,
}

// use std::cell::RefCell;
// use std::rc::Rc;
//
// use crate::builtin::DIALECT_NAME;
// use crate::utils::{trait_id, TraitId};
// use crate::*;
// use tir_macros::operation;
//
// #[operation(name = "const")]
// pub struct ConstOp {
//     #[cfg(attribute = true)]
//     value: Attr,
//     // FIXME: missing type attribute?
// }
//
// #[cfg(test)]
// mod test {
//     use std::any::TypeId;
//
//     use crate::{builtin::*, OpBuilder};
//     use crate::{Context, Op};
//
//     use super::*;
//
//     #[test]
//     fn test_const_op() {
//         assert!(ConstOp::get_operation_name() == "const");
//
//         let context = Context::new();
//         let module = ModuleOp::builder(context.clone()).build();
//         let builder = OpBuilder::new(context.clone(), module.borrow().get_body());
//
//         let attr = Attr::I8(16);
//
//         let constant = ConstOp::builder(context.clone()).value(attr).build();
//
//         builder.borrow_mut().insert(constant.clone());
//         assert_eq!(
//             TryInto::<i8>::try_into(constant.borrow().get_value_attr()).unwrap(),
//             16
//         );
//         let body = module.borrow().get_body().clone();
//         let op = body.borrow().get_operations().first().unwrap().clone();
//         assert_eq!((*op.borrow()).type_id(), TypeId::of::<ConstOp>());
//     }
// }
