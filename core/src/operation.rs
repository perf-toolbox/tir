use crate::utils::CastableMeta;
use crate::{
    AllocId, Attr, ContextRef, ContextWRef, OpAssembly, OpValidator, Printable, RegionRef,
    RegionWRef, Type, Validate, Value,
};
use std::any::Any;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type OpRef = Rc<RefCell<dyn Op>>;

pub trait Op: Any + OpAssembly + Printable + Validate + OpValidator {
    fn get_operation_name(&self) -> &'static str;
    fn get_attrs(&self) -> &HashMap<String, Attr>;
    fn add_attrs(&mut self, attrs: &HashMap<String, Attr>);
    fn get_context(&self) -> ContextRef;
    fn get_parent_region(&self) -> Option<RegionRef>;
    fn get_return_type(&self) -> Option<Type>;
    fn get_return_value(&self) -> Option<Value>;

    fn set_alloc_id(&mut self, id: AllocId);
    fn get_alloc_id(&self) -> AllocId;

    fn get_dialect_id(&self) -> u32;

    fn get_regions(&self) -> OpRegionIter;
    fn has_regions(&self) -> bool;

    #[doc(hidden)]
    fn has_trait(&self, type_id: std::any::TypeId) -> bool;
    #[doc(hidden)]
    fn get_meta(&self) -> &'static linkme::DistributedSlice<[fn() -> CastableMeta]>;
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

pub struct OpRegionIter {
    regions: Vec<RegionRef>,
    index_front: isize,
    index_back: isize,
}

impl OpRegionIter {
    pub fn new(src: &[RegionRef]) -> Self {
        let mut regions = vec![];
        regions.extend_from_slice(src);

        OpRegionIter {
            regions,
            index_front: 0,
            index_back: src.len() as isize - 1,
        }
    }
}

impl Iterator for OpRegionIter {
    type Item = RegionRef;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index_front > self.index_back {
            return None;
        }
        let item = self.regions.get(self.index_front as usize).cloned();
        self.index_front += 1;
        item
    }
}

impl DoubleEndedIterator for OpRegionIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index_front > self.index_back || self.index_back < 0 {
            return None;
        }
        let item = self.regions.get(self.index_back as usize).cloned();
        self.index_front -= 1;
        item
    }
}
