use lpl::ParseResult;

use crate::{Attr, IRFormatter, IRStrStream, OpRef};
use std::any::Any;
use std::collections::HashMap;

type ParseFn<T> = dyn for<'a> Fn(IRStrStream<'a>) -> ParseResult<IRStrStream<'a>, T> + 'static;
pub type OpParseFn = ParseFn<OpRef>;
pub type TyParseFn = ParseFn<HashMap<String, Attr>>;
pub type TyPrintFn = fn(&HashMap<String, Attr>, &mut dyn IRFormatter);

pub struct Dialect {
    name: &'static str,
    id: u32,
    operation_ids: HashMap<&'static str, u32>,
    type_ids: HashMap<&'static str, u32>,
    op_parse_fn: HashMap<u32, Box<OpParseFn>>,
    ty_parse_fn: HashMap<u32, Box<TyParseFn>>,
    ty_print_fn: HashMap<u32, TyPrintFn>,
    ext: Option<Box<dyn Any>>,
}

impl Dialect {
    pub fn new(name: &'static str) -> Dialect {
        Dialect {
            name,
            id: 0,
            operation_ids: HashMap::new(),
            type_ids: HashMap::new(),
            op_parse_fn: HashMap::new(),
            ty_parse_fn: HashMap::new(),
            ty_print_fn: HashMap::new(),
            ext: None,
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

    pub fn add_operation(&mut self, name: &'static str, parser: Box<OpParseFn>) {
        if self
            .operation_ids
            .insert(name, self.operation_ids.len() as u32)
            .is_none()
        {
            self.op_parse_fn
                .insert((self.operation_ids.len() - 1) as u32, parser);
        }
    }

    pub fn get_operation_id(&self, name: &str) -> Option<u32> {
        self.operation_ids.get(name).copied()
    }

    pub fn get_operation_parser(&self, id: u32) -> Option<&OpParseFn> {
        self.op_parse_fn.get(&id).map(|f| f.as_ref())
    }

    pub fn add_type(&mut self, name: &'static str, print_fn: TyPrintFn, parse_fn: Box<TyParseFn>) {
        let id: u32 = self.type_ids.len() as u32;
        self.type_ids.insert(name, id);
        self.ty_print_fn.insert(id, print_fn);
        self.ty_parse_fn.insert(id, parse_fn);
    }

    pub fn get_type_id(&self, name: &str) -> Option<u32> {
        self.type_ids.get(name).copied()
    }

    pub fn get_type_printer(&self, id: u32) -> Option<TyPrintFn> {
        self.ty_print_fn.get(&id).cloned()
    }

    pub fn get_type_parser(&self, id: u32) -> Option<&TyParseFn> {
        self.ty_parse_fn.get(&id).map(|p| p.as_ref())
    }

    pub fn get_similarly_named_op(&self, name: &str) -> Option<&'static str> {
        let mut op_names: Vec<_> = self
            .operation_ids
            .keys()
            .map(|k| (k, strsim::levenshtein(k, name)))
            .collect();

        op_names.sort_by_key(|cand| cand.1);

        op_names
            .first()
            .and_then(|f| if f.1 < 5 { Some(f.0) } else { None })
            .cloned()
    }

    pub fn get_dialect_extension(&self) -> Option<&dyn Any> {
        self.ext.as_ref().map(|e| e.as_ref())
    }

    pub fn get_dialect_extension_mut(&mut self) -> Option<&mut dyn Any> {
        self.ext.as_mut().map(|e| e.as_mut())
    }

    pub fn set_dialect_extension(&mut self, ext: Box<dyn Any>) {
        assert!(self.ext.is_none());
        self.ext = Some(ext);
    }
}
