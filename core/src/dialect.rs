use crate::{ContextRef, OpRef};
use std::collections::HashMap;

pub type ParseFn = fn(ContextRef, &mut &str) -> Result<OpRef, ()>;

pub struct Dialect {
    name: &'static str,
    id: u32,
    operation_ids: HashMap<&'static str, u32>,
    type_ids: HashMap<&'static str, u32>,
    parse_fn: HashMap<u32, ParseFn>,
}

impl Dialect {
    pub fn new(name: &'static str) -> Dialect {
        Dialect {
            name,
            id: 0,
            operation_ids: HashMap::new(),
            type_ids: HashMap::new(),
            parse_fn: HashMap::new(),
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

    pub fn add_operation(&mut self, name: &'static str, parser: ParseFn) {
        if self
            .operation_ids
            .insert(name, self.operation_ids.len().try_into().unwrap())
            .is_none()
        {
            self.parse_fn
                .insert((self.operation_ids.len() - 1).try_into().unwrap(), parser);
        }
    }

    pub fn get_operation_id(&self, name: &str) -> Option<u32> {
        self.operation_ids.get(name).cloned()
    }

    pub fn get_operation_parser(&self, id: u32) -> Option<ParseFn> {
        self.parse_fn.get(&id).cloned()
    }

    pub fn add_type(&mut self, name: &'static str) {
        self.type_ids
            .insert(name, self.type_ids.len().try_into().unwrap());
    }

    pub fn get_type_id(&self, name: &'static str) -> u32 {
        *self.type_ids.get(name).unwrap()
    }
}
