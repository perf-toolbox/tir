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
}

pub trait Ty {
    fn get_type_name() -> &'static str;
}
