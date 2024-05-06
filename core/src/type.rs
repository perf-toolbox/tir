use std::collections::HashMap;
use std::sync::Arc;

use crate::Attr;
use crate::ContextRef;
use crate::ContextWRef;

#[derive(Debug, Clone)]
pub struct Type {
    context: ContextWRef,
    dialect_id: u32,
    type_id: u32,
    attrs: HashMap<String, Attr>,
}

impl Type {
    pub fn new(
        context: ContextRef,
        dialect_id: u32,
        type_id: u32,
        attrs: HashMap<String, Attr>,
    ) -> Self {
        Type {
            context: Arc::downgrade(&context),
            dialect_id,
            type_id,
            attrs,
        }
    }

    pub fn get_context(&self) -> Option<ContextRef> {
        self.context.upgrade()
    }

    pub fn get_dialect_id(&self) -> u32 {
        self.dialect_id
    }

    pub fn get_type_id(&self) -> u32 {
        self.type_id
    }

    pub fn get_attrs(&self) -> &HashMap<String, Attr> {
        &self.attrs
    }

    pub fn isa<T: Ty>(&self) -> bool {
        let context = self.context.upgrade().unwrap();
        let dialect = context.get_dialect_by_name(T::get_dialect_name()).unwrap();
        if dialect.get_id() != self.dialect_id {
            return false;
        }
        let type_id = dialect.get_type_id(T::get_type_name());
        type_id == self.type_id
    }
}

pub trait Ty {
    fn get_type_name() -> &'static str;
    fn get_dialect_name() -> &'static str;
}
