use crate::{BlockRef, ContextRef, Op, OpRef};
use std::cell::RefCell;
use std::rc::Rc;

pub struct InsertionPoint {
    block: BlockRef,
    index: usize,
}

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

    fn set_insertion_point_to_start(&mut self, block: BlockRef) {
        self.insertion_point.block = block;
        self.insertion_point.index = 0;
    }
}

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

    pub fn insert_generic(&self, op: &OpRef) {
        self.0.borrow_mut().insert(op);
    }

    pub fn get_context(&self) -> ContextRef {
        self.0.borrow().context.clone()
    }

    pub fn set_insertion_point_to_start(&self, block: BlockRef) {
        self.0.borrow_mut().set_insertion_point_to_start(block);
    }
}
