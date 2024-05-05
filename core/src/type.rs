use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::Attr;
use crate::Context;

#[derive(Debug, Clone)]
pub struct Type {
    context: Rc<RefCell<Context>>,
    dialect_id: u32,
    type_id: u32,
    attrs: HashMap<String, Attr>,
}

impl Type {
    pub fn new(
        context: Rc<RefCell<Context>>,
        dialect_id: u32,
        type_id: u32,
        attrs: HashMap<String, Attr>,
    ) -> Self {
        Type {
            context,
            dialect_id,
            type_id,
            attrs,
        }
    }

    pub fn get_context(&self) -> Rc<RefCell<Context>> {
        self.context.clone()
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

pub trait Ty: PartialEq {
    fn get_type_name() -> &'static str;
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        // We compare contexts as raw pointers, so two structurally identical
        // datatypes in separate contexts are treated as unequal.
        self.context.as_ptr() == other.context.as_ptr()
            && self.attrs == other.attrs
            && self.type_id == other.type_id
            && self.dialect_id == other.dialect_id
    }
}
