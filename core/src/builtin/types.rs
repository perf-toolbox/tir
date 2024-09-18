use crate::Printable;
use crate::{Attr, ContextRef, Ty, TyAssembly, Type};
use std::collections::HashMap;
use tir_macros::{dialect_type, dialect_type_with_extensions};

use crate as tir_core;

use crate::builtin::DIALECT_NAME;

dialect_type_with_extensions!(FuncType);
dialect_type!(VoidType);
dialect_type!(IntType);

impl TyAssembly for VoidType {
    fn print_assembly(
        _attrs: &HashMap<String, tir_core::Attr>,
        fmt: &mut dyn tir_core::IRFormatter,
    ) {
        fmt.write_direct("void");
    }

    fn parse_assembly<'a>() -> BoxedParser<'a, IRStrStream<'a>, std::collections::HashMap<String, tir_core::Attr>> {
        // tir_core::parser::skip_attrs(input)
        todo!()
    }
}

impl FuncType {
    fn get_inputs_attr_name() -> &'static str {
        "inputs"
    }

    fn get_return_attr_name() -> &'static str {
        "return"
    }

    pub fn build(context: ContextRef, input_types: &[Type], return_type: Type) -> FuncType {
        let mut attrs = HashMap::new();

        attrs.insert(
            FuncType::get_inputs_attr_name().to_string(),
            Attr::TypeArray(input_types.to_vec()),
        );
        attrs.insert(
            FuncType::get_return_attr_name().to_string(),
            Attr::Type(return_type),
        );

        let dialect = context.get_dialect_by_name(DIALECT_NAME).unwrap();
        // we are sure the type exists, because we are the type!
        let type_id = dialect.get_type_id(FuncType::get_type_name()).unwrap();
        let r#type = Type::new(context.clone(), dialect.get_id(), type_id, attrs);

        FuncType { r#type }
    }

    pub fn get_inputs(&self) -> &[Type] {
        match self
            .r#type
            .get_attrs()
            .get(Self::get_inputs_attr_name())
            .as_ref()
            .unwrap()
        {
            Attr::TypeArray(array) => array,
            _ => panic!("Expected 'inputs' to be a TypeArray"),
        }
    }

    pub fn get_return(&self) -> &Type {
        match self
            .r#type
            .get_attrs()
            .get(Self::get_return_attr_name())
            .as_ref()
            .unwrap()
        {
            Attr::Type(type_) => type_,
            _ => panic!("Expected 'return' to be a Type"),
        }
    }
}

impl VoidType {
    pub fn build(context: ContextRef) -> VoidType {
        let dialect = context.get_dialect_by_name(DIALECT_NAME).unwrap();
        // we are sure the type exists, because we are the type!
        let type_id = dialect.get_type_id(VoidType::get_type_name()).unwrap();
        let r#type = Type::new(context, dialect.get_id(), type_id, HashMap::new());

        VoidType { r#type }
    }
}

impl IntType {
    fn get_bits_attr_name() -> &'static str {
        "bits"
    }

    pub fn build(context: ContextRef, bits: u32) -> IntType {
        let mut attrs = HashMap::new();

        attrs.insert(IntType::get_bits_attr_name().to_string(), Attr::U32(bits));

        let dialect = context.get_dialect_by_name(DIALECT_NAME).unwrap();
        // we are sure the type exists, because we are the type!
        let type_id = dialect.get_type_id(IntType::get_type_name()).unwrap();
        let r#type = Type::new(context.clone(), dialect.get_id(), type_id, attrs);

        IntType { r#type }
    }

    pub fn get_bits(&self) -> u32 {
        match self
            .r#type
            .get_attrs()
            .get(Self::get_bits_attr_name())
            .as_ref()
            .unwrap()
        {
            Attr::U32(bits) => *bits,
            _ => panic!("Expected 'bits' to be u32"),
        }
    }
}

impl TyAssembly for IntType {
    fn print_assembly(
        attrs: &HashMap<String, tir_core::Attr>,
        fmt: &mut dyn tir_core::IRFormatter,
    ) {
        fmt.write_direct("int<");
        if let Some(bits) = attrs.get("bits") {
            let bits_int: u32 = (bits.clone()).try_into().unwrap();
            fmt.write_direct(&bits_int.to_string());
        }
        fmt.write_direct(">");
    }

    fn parse_assembly<'a>() -> BoxedParser<'a, IRStrStream<'a>, HashMap<String, Attr>>
    where
        Self: Sized,
    {
        // tir_core::parser::parse_int_bits(input)
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Context, Printable, StringPrinter, Type};

    use super::*;

    #[test]
    fn type_casts() {
        let context = Context::new();

        let ty = VoidType::build(context.clone());
        let mut printer = StringPrinter::new();
        ty.print(&mut printer);
        assert_eq!("!void", &printer.get());
        let ty: Type = ty.into();
        assert!(ty.isa::<VoidType>());
        assert!(VoidType::try_from(ty.clone()).is_ok());
        assert!(FuncType::try_from(ty.clone()).is_err());
        assert!(IntType::try_from(ty.clone()).is_err());

        let ty = FuncType::build(context.clone(), &[], ty);
        let ty: Type = ty.into();
        assert!(ty.isa::<FuncType>());
        assert!(VoidType::try_from(ty.clone()).is_err());
        assert!(FuncType::try_from(ty.clone()).is_ok());
        assert!(IntType::try_from(ty.clone()).is_err());

        let ty = IntType::build(context.clone(), 8);
        let ty: Type = ty.into();
        assert!(ty.isa::<IntType>());
        assert!(VoidType::try_from(ty.clone()).is_err());
        assert!(FuncType::try_from(ty.clone()).is_err());
        assert!(IntType::try_from(ty.clone()).is_ok());
        assert_eq!(IntType::try_from(ty.clone()).unwrap().get_bits(), 8);
    }
}
