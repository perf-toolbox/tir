use crate::{BlockRef, ContextRef, Op, OpRef};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct InsertionPoint {
    block: BlockRef,
    index: usize,
}

#[derive(Debug)]
pub struct OpBuilderImpl {
    context: ContextRef,
    insertion_point: InsertionPoint,
}

impl OpBuilderImpl {
    fn new(context: ContextRef, block: BlockRef) -> Rc<RefCell<Self>> {
        let insertion_point = InsertionPoint { block, index: 0 };
        Rc::new(RefCell::new(Self {
            context,
            insertion_point,
        }))
    }

    fn insert(&mut self, op: &OpRef) {
        self.insertion_point
            .block
            .insert(self.insertion_point.index, op);
        self.insertion_point.index += 1;
    }

    fn erase(&mut self, op: &OpRef) {
        if let Some(region) = op.borrow().get_parent_region() {
            if let Some(blk) = region.find_op_block(op) {
                blk.erase(op);
            }
        }
    }

    fn set_insertion_point_to_start(&mut self, block: BlockRef) {
        self.insertion_point.block = block;
        self.insertion_point.index = 0;
    }

    fn set_insertion_point_after<T: Op + ?Sized>(&mut self, op: &Rc<RefCell<T>>) {
        let parent = op.borrow().get_parent_region().unwrap();
        let (block, id) = parent
            .iter()
            .find_map(|b| b.find(op.borrow().get_alloc_id()).map(|id| (b, id)))
            .unwrap();

        self.insertion_point.block = block;
        self.insertion_point.index = id + 1;
    }
}

#[derive(Debug, Clone)]
pub struct OpBuilder(Rc<RefCell<OpBuilderImpl>>);

impl OpBuilder {
    pub fn new(context: ContextRef, block: BlockRef) -> Self {
        Self(OpBuilderImpl::new(context, block))
    }

    pub fn insert<T>(&self, op: &Rc<RefCell<T>>)
    where
        T: Op,
    {
        let op: OpRef = op.clone();
        self.0.borrow_mut().insert(&op);
    }

    pub fn erase(&self, op: &OpRef) {
        self.0.borrow_mut().erase(&op);
    }

    pub fn insert_generic(&self, op: &OpRef) {
        self.0.borrow_mut().insert(op);
    }

    pub fn get_context(&self) -> ContextRef {
        self.0.borrow().context.clone()
    }

    pub fn set_insertion_point_to_start(&self, block: BlockRef) {
        self.0.borrow_mut().set_insertion_point_to_start(block);
    }

    pub fn set_insertion_point_after<T: Op + ?Sized>(&self, op: &Rc<RefCell<T>>) {
        self.0.borrow_mut().set_insertion_point_after(op);
    }
}
