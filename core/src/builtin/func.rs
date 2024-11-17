use crate::builtin::DIALECT_NAME;
use crate::*;
use lpl::combinators::{literal, separated_ignore, spaced};
use lpl::ParseResult;
use lpl::{ParseStream, Parser};
use parser::{identifier, region_with_blocks, sym_name, Parsable};
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

fn single_arg<'a>() -> impl Parser<'a, IRStrStream<'a>, (&'a str, Type)> {
    literal("%")
        .and_then(identifier())
        .and_then(spaced(literal(":")))
        .and_then(Type::parse)
        .map(|(((_, name), _), ty)| (name, ty))
        .label("single_arg")
}

fn signature<'a>() -> impl Parser<'a, IRStrStream<'a>, (Vec<&'a str>, FuncType)> {
    spaced(literal("("))
        .and_then(separated_ignore(single_arg(), spaced(literal(",")).void()).label("arg list"))
        .and_then(spaced(literal(")")))
        .map(|((_, args), _)| args)
        .and_then(spaced(literal("->")))
        .and_then(Type::parse)
        .map_with(|((args, _), return_type), extra| {
            let state = extra.unwrap();
            let context = state.context();
            let (names, input_types): (Vec<&'a str>, Vec<Type>) = args.iter().cloned().unzip();

            let func_ty = FuncType::build(context.clone(), &input_types, return_type);

            (names, func_ty)
        })
        .label("signature")
}

impl OpAssembly for FuncOp {
    fn parse_assembly(input: IRStrStream) -> ParseResult<IRStrStream, OpRef>
    where
        Self: Sized,
    {
        let parser = sym_name().and_then(signature());

        let state = input.get_extra().cloned().unwrap();

        let ((func_name, (arg_names, func_ty)), ni) = parser.parse(input)?;

        let arg_names: Vec<_> = arg_names.into_iter().map(|n| n.to_owned()).collect();
        let arg_types = func_ty.get_inputs().to_vec();

        state.set_deferred_types(arg_types);
        state.set_deferred_names(arg_names);

        let (region, ni) = region_with_blocks().parse(ni.unwrap())?;

        let context = state.context();

        let func = FuncOp::builder(&context)
            .sym_name(Attr::String(func_name.into()))
            .func_type(Attr::Type(func_ty.into()))
            .body(region)
            .build();
        Ok((func, ni))
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
