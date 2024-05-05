use std::rc::{Rc, Weak};

use crate::ContextRef;

pub type RegionRef = Rc<Region>;
pub type RegionWRef = Weak<Region>;

pub struct Region {}

impl Region {
    pub fn with_single_block(_context: &ContextRef) -> RegionRef {
        Rc::new(Region {})
    }
}
