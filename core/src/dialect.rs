use crate::{Op, Operation};
use std::collections::HashMap;
use std::rc::Rc;

pub type DispatchFn = Box<dyn Fn(Operation) -> Option<Box<dyn Op>>>;

pub struct Dialect {
    name: &'static str,
    id: u32,
    operation_ids: HashMap<&'static str, u32>,
    type_ids: HashMap<&'static str, u32>,
    _dispatch_fn: Rc<DispatchFn>,
}

impl Dialect {
    pub fn new(name: &'static str, dispatch_fn: DispatchFn) -> Dialect {
        Dialect {
            name,
            id: 0,
            operation_ids: HashMap::new(),
            type_ids: HashMap::new(),
            _dispatch_fn: Rc::new(dispatch_fn),
        }
    }

    pub fn set_id(&mut self, id: u32) {
        if self.id != 0 {
            panic!("Dialect ID already set");
        }
        self.id = id;
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_name(&self) -> &'static str {
        self.name
    }

    pub fn add_operation(&mut self, name: &'static str) {
        self.operation_ids
            .insert(name, self.operation_ids.len().try_into().unwrap());
    }

    pub fn get_operation_id(&self, name: &'static str) -> u32 {
        *self.operation_ids.get(name).unwrap()
    }

    pub fn add_type(&mut self, name: &'static str) {
        self.type_ids
            .insert(name, self.type_ids.len().try_into().unwrap());
    }

    pub fn get_type_id(&self, name: &'static str) -> u32 {
        *self.type_ids.get(name).unwrap()
    }
}
