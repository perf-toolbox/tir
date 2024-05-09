use crate::{AllocId, Assembly, Attr, ContextRef, ContextWRef, RegionRef, RegionWRef, Type};
use std::any::Any;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type OpRef = Rc<RefCell<dyn Op>>;

pub trait Op: Any + Assembly {
    fn get_operation_name(&self) -> &'static str;
    fn get_attrs(&self) -> &HashMap<String, Attr>;
    fn get_context(&self) -> ContextRef;
    fn get_parent_region(&self) -> Option<RegionRef>;
    fn get_return_type(&self) -> Option<Type>;

    fn set_alloc_id(&mut self, id: AllocId);
    fn get_alloc_id(&self) -> AllocId;

    fn get_dialect_id(&self) -> u32;
}

#[derive(Debug, Clone)]
pub struct OpImpl {
    pub context: ContextWRef,
    pub dialect_id: u32,
    pub operation_id: u32,
    pub alloc_id: AllocId,
    pub parent_region: Option<RegionWRef>,
    pub attrs: HashMap<String, Attr>,
}
