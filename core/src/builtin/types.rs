use crate::{Attr, ContextRef, Ty, TyAssembly, Type};
use std::collections::HashMap;
use tir_macros::dialect_type;

use crate as tir_core;

use crate::builtin::DIALECT_NAME;

dialect_type!(FuncType);
dialect_type!(VoidType);
dialect_type!(IntegerType);

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
        let type_id = dialect.get_type_id(FuncType::get_type_name());
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
        let type_id = dialect.get_type_id(VoidType::get_type_name());
        let r#type = Type::new(context, dialect.get_id(), type_id, HashMap::new());

        VoidType { r#type }
    }
}

impl IntegerType {
    fn get_bit_width_attr_name() -> &'static str {
        "bit_width"
    }

    pub fn build(context: ContextRef, bit_width: u32) -> IntegerType {
        let mut attrs = HashMap::new();

        attrs.insert(
            IntegerType::get_bit_width_attr_name().to_string(),
            Attr::U32(bit_width),
        );

        let dialect = context.get_dialect_by_name(DIALECT_NAME).unwrap();
        let type_id = dialect.get_type_id(IntegerType::get_type_name());
        let r#type = Type::new(context.clone(), dialect.get_id(), type_id, attrs);

        IntegerType { r#type }
    }

    pub fn get_bit_width(&self) -> u32 {
        match self
            .r#type
            .get_attrs()
            .get(Self::get_bit_width_attr_name())
            .as_ref()
            .unwrap()
        {
            Attr::U32(bit_width) => *bit_width,
            _ => panic!("Expected 'bit_width' to be u32"),
        }
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
        assert!(IntegerType::try_from(ty.clone()).is_err());

        let ty = FuncType::build(context.clone(), &[], ty);
        let ty: Type = ty.into();
        assert!(ty.isa::<FuncType>());
        assert!(VoidType::try_from(ty.clone()).is_err());
        assert!(FuncType::try_from(ty.clone()).is_ok());
        assert!(IntegerType::try_from(ty.clone()).is_err());

        let ty = IntegerType::build(context.clone(), 8);
        let ty: Type = ty.into();
        assert!(ty.isa::<IntegerType>());
        assert!(VoidType::try_from(ty.clone()).is_err());
        assert!(FuncType::try_from(ty.clone()).is_err());
        assert!(IntegerType::try_from(ty.clone()).is_ok());
        assert_eq!(
            IntegerType::try_from(ty.clone()).unwrap().get_bit_width(),
            8
        );
    }
}
