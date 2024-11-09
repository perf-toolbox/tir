use crate::builtin::DIALECT_NAME;
use crate::*;
use lpl::ParseResult;
use lpl::{Parser, ParseStream};
use tir_macros::{op_implements, Op, OpAssembly, OpValidator};

use crate as tir_core;

use super::FuncType;

#[derive(Op, OpValidator)]
#[operation(name = "func", dialect = builtin, known_attrs(sym_name: String, func_type: Type))]
pub struct FuncOp {
    #[region]
    body: RegionRef,
    r#impl: OpImpl,
}

/// Return from a function
#[derive(Op, OpValidator, OpAssembly)]
#[operation(name = "return", dialect = builtin)]
pub struct ReturnOp {
    // TODO add an optional return value
    r#impl: OpImpl,
}

#[op_implements(dialect = builtin)]
impl Terminator for ReturnOp {}

// fn single_arg<'s>(input: &mut ParseStream<'s>) -> AsmPResult<(&'s str, Type)> {
//     (
//         preceded("%", identifier),
//         preceded((space0, ":", space0), Type::parse),
//     )
//         .parse_next(input)
// }
//
// fn signature<'s>(input: &mut ParseStream<'s>) -> AsmPResult<(Vec<&'s str>, FuncType)> {
//     let braces: Vec<(&'s str, Type)> = delimited(
//         (space0, "(", space0),
//         separated(0.., single_arg, (space0, ",", space0)),
//         (space0, ")", space0),
//     )
//     .parse_next(input)?;
//
//     let (names, input_types): (Vec<&'s str>, Vec<Type>) = braces.iter().cloned().unzip();
//
//     let return_type = preceded((space0, "->", space0), Type::parse).parse_next(input)?;
//
//     let context = input.state.get_context();
//     let func_ty = FuncType::build(context, &input_types, return_type);
//
//     Ok((names, func_ty))
// }

impl OpAssembly for FuncOp {
    fn parse_assembly(input: IRStrStream) -> ParseResult<IRStrStream, OpRef>
    where
        Self: Sized,
    {
        todo!()
        // let sym_name_str = trace("sym_name", preceded(space0, sym_name)).parse_next(input)?;
        //
        // let (arg_names, func_ty) = signature.parse_next(input)?;
        //
        // let arg_names: Vec<_> = arg_names.into_iter().map(|n| n.to_owned()).collect();
        // let arg_types = func_ty.get_inputs().to_vec();
        //
        // input.state.set_deferred_types(arg_types);
        // input.state.set_deferred_names(arg_names);
        //
        // let region = region_with_blocks.parse_next(input)?;
        //
        // let context = input.state.get_context();
        //
        // let func = FuncOp::builder(&context)
        //     .sym_name(Attr::String(sym_name_str.into()))
        //     .func_type(Attr::Type(func_ty.into()))
        //     .body(region)
        //     .build();
        //
        // Ok(func)
    }

    fn print_assembly(&self, fmt: &mut dyn IRFormatter) {
        fmt.write_direct(&format!(
            "@{}",
            TryInto::<String>::try_into(self.get_sym_name_attr().clone()).unwrap()
        ));

        let func_ty: FuncType = self.get_func_type_attr().clone().try_into().unwrap();

        fmt.write_direct("(");

        let types: Vec<_> = self
            .get_body_region()
            .first()
            .unwrap()
            .get_args()
            .map(|arg| {
                let mut printer = StringPrinter::new();
                printer.write_direct(&format!("%{}: ", &arg.get_name()));
                arg.get_type().print(&mut printer);
                printer.get()
            })
            .collect();
        print_comma_separated(fmt, &types);
        fmt.write_direct(")");
        fmt.write_direct(" -> ");
        func_ty.get_return().print(fmt);
        fmt.write_direct(" ");
        print_region(fmt, &self.get_body_region());
    }
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
