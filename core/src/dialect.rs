use crate::parser::{AsmPResult, ParseStream};
use crate::{Attr, IRFormatter, OpRef};
use std::collections::HashMap;

type ParseFn<T> = fn(&mut ParseStream) -> AsmPResult<T>;
pub type OpParseFn = ParseFn<OpRef>;
pub type TyParseFn = ParseFn<HashMap<String, Attr>>;
pub type TyPrintFn = fn(&HashMap<String, Attr>, &mut dyn IRFormatter);

pub struct Dialect {
    name: &'static str,
    id: u32,
    operation_ids: HashMap<&'static str, u32>,
    type_ids: HashMap<&'static str, u32>,
    op_parse_fn: HashMap<u32, OpParseFn>,
    ty_parse_fn: HashMap<u32, TyParseFn>,
    ty_print_fn: HashMap<u32, TyPrintFn>,
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

    pub fn add_operation(&mut self, name: &'static str, parser: OpParseFn) {
        if self
            .operation_ids
            .insert(name, self.operation_ids.len().try_into().unwrap())
            .is_none()
        {
            self.op_parse_fn
                .insert((self.operation_ids.len() - 1).try_into().unwrap(), parser);
        }
    }

    pub fn get_operation_id(&self, name: &str) -> Option<u32> {
        self.operation_ids.get(name).cloned()
    }

    pub fn get_operation_parser(&self, id: u32) -> Option<OpParseFn> {
        self.op_parse_fn.get(&id).cloned()
    }

    pub fn add_type(&mut self, name: &'static str, print_fn: TyPrintFn, parse_fn: TyParseFn) {
        let id: u32 = self.type_ids.len().try_into().unwrap();
        self.type_ids.insert(name, id);
        self.ty_print_fn.insert(id, print_fn);
        self.ty_parse_fn.insert(id, parse_fn);
    }

    pub fn get_type_id(&self, name: &str) -> u32 {
        *self.type_ids.get(name).unwrap()
    }

    pub fn get_type_printer(&self, id: u32) -> Option<TyPrintFn> {
        self.ty_print_fn.get(&id).cloned()
    }

    pub fn get_type_parser(&self, id: u32) -> Option<TyParseFn> {
        self.ty_parse_fn.get(&id).cloned()
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
}
