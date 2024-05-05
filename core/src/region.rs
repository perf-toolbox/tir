use std::{
    cell::RefCell,
    rc::{Rc, Weak},
    sync::Arc,
};

use crate::{AllocId, ContextRef, ContextWRef, OpRef};

pub type RegionRef = Rc<Region>;
pub type RegionWRef = Weak<Region>;
pub type BlockRef = Rc<Block>;

struct BlockImpl {
    parent_region: RegionWRef,
    operations: Vec<AllocId>,
}

impl BlockImpl {
    fn new(parent_region: RegionWRef) -> Self {
        Self {
            parent_region,
            operations: vec![],
        }
    }

    fn push(&mut self, op: AllocId) {
        self.operations.push(op);
    }

    fn insert(&mut self, index: usize, op: AllocId) {
        self.operations.insert(index, op);
    }

    fn get_parent_region(&self) -> RegionRef {
        self.parent_region.upgrade().unwrap()
    }

    fn get_context(&self) -> ContextRef {
        let parent = self.parent_region.upgrade().unwrap();
        parent.get_context()
    }

    fn first(&self) -> Option<OpRef> {
        let context = self.get_context();
        self.operations.first().map(|id| context.get_op(*id))?
    }
}

pub struct Block {
    r#impl: RefCell<BlockImpl>,
}

impl Block {
    pub fn empty(parent: &RegionRef) -> BlockRef {
        Rc::new(Block {
            r#impl: RefCell::new(BlockImpl::new(Rc::downgrade(parent))),
        })
    }

    pub fn push(&self, op: &OpRef) {
        self.r#impl.borrow_mut().push(op.borrow().get_alloc_id());
    }

    pub fn insert(&self, index: usize, op: &OpRef) {
        self.r#impl
            .borrow_mut()
            .insert(index, op.borrow().get_alloc_id());
    }

    pub fn get_parent_region(&self) -> RegionRef {
        self.r#impl.borrow().get_parent_region()
    }

    pub fn first(&self) -> Option<OpRef> {
        self.r#impl.borrow().first()
    }
}

struct RegionImpl {
    context: ContextWRef,
    parent_op: AllocId,
    blocks: Vec<BlockRef>,
}

impl RegionImpl {
    fn new(context: ContextWRef) -> RegionImpl {
        RegionImpl {
            context,
            parent_op: AllocId::default(),
            blocks: vec![],
        }
    }

    fn get_context(&self) -> ContextRef {
        self.context.upgrade().unwrap()
    }
}

pub struct Region {
    r#impl: RefCell<RegionImpl>,
}

impl Region {
    pub fn empty(context: &ContextRef) -> RegionRef {
        Rc::new(Region {
            r#impl: RefCell::new(RegionImpl::new(Arc::downgrade(context))),
        })
    }

    pub fn with_single_block(context: &ContextRef) -> RegionRef {
        let region = Region::empty(context);
        let block = Block::empty(&region);
        region.add_block(block);

        region
    }

    pub fn get_context(&self) -> ContextRef {
        self.r#impl.borrow().get_context()
    }

    pub fn add_block(&self, block: BlockRef) {
        self.r#impl.borrow_mut().blocks.push(block);
    }

    pub fn get_parent_op(&self) -> OpRef {
        let context = self.r#impl.borrow().context.upgrade().unwrap();
        context.get_op(self.r#impl.borrow().parent_op).unwrap()
    }

    pub fn first(&self) -> Option<BlockRef> {
        self.r#impl.borrow().blocks.first().cloned()
    }
}
