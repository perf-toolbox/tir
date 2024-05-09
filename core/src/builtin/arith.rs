use crate::assembly::parser::Parseable;
use crate::builtin::value::AnyValue;
use crate::builtin::DIALECT_NAME;
use crate::OpAssembly;
use crate::Printable;
use crate::{Op, OpImpl, OpRef, Type};
use tir_macros::{Op, OpAssembly};
use winnow::Parser;

use crate as tir_core;

#[derive(Op, OpAssembly, Clone)]
#[operation(name = "const", known_attrs(value: IntegerAttr))]
pub struct ConstOp {
    #[ret_type]
    return_type: Type,
    r#impl: OpImpl,
}

impl From<ConstOp> for AnyValue {
    fn from(c: ConstOp) -> Self {
        Self {
            op_id: c.get_alloc_id(),
            ty: c.get_return_type().unwrap(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parse_ir;
    use crate::Attr;
    use crate::Context;
    use crate::OpBuilder;
    use crate::Printable;
    use crate::StringPrinter;
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
            .value(attr)
            .return_type(ret_type.into())
            .build();

        constant.borrow().get_context();
        module.borrow().get_context();

        let mut printer = StringPrinter::new();
        constant.borrow().print(&mut printer);
        assert_eq!(printer.get(), "const attrs = {value = <i8: 16>} -> !void");

        builder.insert(&constant);
        assert_eq!(
            TryInto::<i8>::try_into(constant.borrow().get_value_attr().clone()).unwrap(),
            16
        );
        let body = module.borrow().get_body().clone();
        let op = body.first().unwrap();
        assert_eq!((*op.borrow()).type_id(), TypeId::of::<ConstOp>());
        let v1: AnyValue = From::<ConstOp>::from((constant.borrow()).clone());
        let v2: Value<VoidType> = TryInto::<Value<VoidType>>::try_into(v1.clone()).unwrap();
        assert_eq!(
            op.borrow().get_alloc_id(),
            v2.get_defining_op().borrow().get_alloc_id()
        );
        let v3: AnyValue = From::<Value<VoidType>>::from(v2);
        assert_eq!(v1, v3);
        assert_eq!(
            op.borrow().get_alloc_id(),
            v3.get_defining_op().borrow().get_alloc_id()
        );
    }

    #[test]
    fn parse_const() {
        let ir = "
        module {
            const attrs = {value = <i8: 16>} -> !void 
        }
        ";

        let context = Context::new();
        let module = parse_ir(context.clone(), ir).expect("module");

        let module = utils::op_cast::<ModuleOp>(module).unwrap();

        assert_eq!(
            (*module.borrow().get_body().first().unwrap().borrow()).type_id(),
            TypeId::of::<ConstOp>()
        );
    }
}
