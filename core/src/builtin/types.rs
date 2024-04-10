use crate::{Attr, Context, Ty, Type};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::builtin::DIALECT_NAME;

pub struct FuncType {
    r#type: Type,
}

impl FuncType {
    fn get_inputs_attr_name() -> &'static str {
        "inputs"
    }

    fn get_return_attr_name() -> &'static str {
        "return"
    }

    pub fn new(r#type: Type) -> Self {
        // TODO check type is correct
        Self { r#type }
    }

    pub fn build(
        context: Rc<RefCell<Context>>,
        input_types: &[Type],
        return_type: Type,
    ) -> FuncType {
        let mut attrs = HashMap::new();

        attrs.insert(
            FuncType::get_inputs_attr_name().to_string(),
            Attr::TypeArray(input_types.into_iter().cloned().collect()),
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
        match self.r#type.get_attrs().get("inputs").as_ref().unwrap() {
            Attr::TypeArray(array) => array,
            _ => panic!("Expected 'inputs' to be a TypeArray"),
        }
    }

    pub fn get_return(&self) -> &Type {
        match self.r#type.get_attrs().get("return").as_ref().unwrap() {
            Attr::Type(type_) => type_,
            _ => panic!("Expected 'return' to be a Type"),
        }
    }
}

impl Ty for FuncType {
    fn get_type_name() -> &'static str {
        "func"
    }
}

pub struct VoidType {
    r#type: Type,
}

impl VoidType {
    pub fn new(r#type: Type) -> Self {
        // TODO check type is correct
        Self { r#type }
    }
}

impl Ty for VoidType {
    fn get_type_name() -> &'static str {
        "void"
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

impl Into<Type> for VoidType {
    fn into(self) -> Type {
        self.r#type
    }
}
