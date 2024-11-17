use crate::builtin::DIALECT_NAME;
use crate::OpAssembly;
use crate::Printable;
use crate::{Op, OpImpl, Type};
use lpl::{ParseStream, Parser};
use tir_macros::{Op, OpAssembly, OpValidator};

use crate as tir_core;

#[derive(Op, OpAssembly, Clone, OpValidator)]
#[operation(name = "const", dialect = builtin, known_attrs(value: IntegerAttr))]
pub struct ConstOp {
    #[ret_type]
    return_type: Type,
    r#impl: OpImpl,
}

#[cfg(test)]
mod test {
    use crate::parse_ir;
    use crate::Attr;
    use crate::Context;
    use crate::OpBuilder;
    use crate::Printable;
    use crate::StringPrinter;
    use crate::Value;
    use crate::{builtin::*, utils};
    use std::any::TypeId;

    use super::*;

    #[test]
    fn test_const_op() {
        assert!(ConstOp::get_operation_name() == "const");

        let context = Context::new();
        let module = ModuleOp::builder(&context).build();
        let builder = OpBuilder::new(context.clone(), module.borrow().get_body());

        let attr = Attr::I8(16);

        // FIXME: this cannot be void
        let ret_type = VoidType::build(context.clone());
        let constant = ConstOp::builder(&context)
            .value(attr.clone())
            .return_type(ret_type.clone().into())
            .build();

        constant.borrow().get_context();
        module.borrow().get_context();

        let mut printer = StringPrinter::new();
        constant.borrow().print(&mut printer);
        assert_eq!(printer.get(), "const attrs = {value = <i8: 16>} -> !void\n");

        builder.insert(&constant);
        assert_eq!(
            TryInto::<i8>::try_into(constant.borrow().get_value_attr().clone()).unwrap(),
            16
        );
        let body = module.borrow().get_body().clone();
        let op = body.first().unwrap();
        assert_eq!((*op.borrow()).type_id(), TypeId::of::<ConstOp>());

        let other_constant = ConstOp::builder(&context)
            .value(attr)
            .return_type(ret_type.into())
            .build();
        let value = constant.borrow().get_return_value().unwrap();
        let other_value = other_constant.borrow().get_return_value().unwrap();

        assert_ne!(value, other_value);
        let v2: Result<Value<VoidType>, ()> = value.try_cast();
        assert!(v2.is_ok());
    }

    #[test]
    fn parse_const() {
        let ir = "
        module {
            const attrs = {value = <i8: 16>} -> !void
        }
        ";

        let context = Context::new();
        let module = parse_ir(context.clone(), ir, "-").expect("module");

        let module = utils::op_cast::<ModuleOp>(module).unwrap();

        assert_eq!(
            (*module.borrow().get_body().first().unwrap().borrow()).type_id(),
            TypeId::of::<ConstOp>()
        );
    }
}
