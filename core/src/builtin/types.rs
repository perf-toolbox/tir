use crate::{Attr, Context, Ty, Type};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tir_macros::dialect_type;

use crate::builtin::DIALECT_NAME;

dialect_type!(FuncType);
dialect_type!(VoidType);

impl FuncType {
    fn get_inputs_attr_name() -> &'static str {
        "inputs"
    }

    fn get_return_attr_name() -> &'static str {
        "return"
    }

    pub fn build(
        context: Rc<RefCell<Context>>,
        input_types: &[Type],
        return_type: Type,
    ) -> FuncType {
        let mut attrs = HashMap::new();

        attrs.insert(
            FuncType::get_inputs_attr_name().to_string(),
            Attr::TypeArray(input_types.to_vec()),
        );
        attrs.insert(
            FuncType::get_return_attr_name().to_string(),
            Attr::Type(return_type),
        );

        let dialect = context.borrow().get_dialect_by_name(DIALECT_NAME).unwrap();
        let type_id = dialect.borrow().get_type_id(FuncType::get_type_name());
        let r#type = Type::new(context.clone(), dialect.borrow().get_id(), type_id, attrs);

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
    pub fn build(context: Rc<RefCell<Context>>) -> VoidType {
        let dialect = context.borrow().get_dialect_by_name(DIALECT_NAME).unwrap();
        let type_id = dialect.borrow().get_type_id(VoidType::get_type_name());
        let r#type = Type::new(context, dialect.borrow().get_id(), type_id, HashMap::new());

        VoidType { r#type }
    }
}
