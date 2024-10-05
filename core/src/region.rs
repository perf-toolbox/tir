use std::{
    cell::RefCell,
    iter::zip,
    rc::{Rc, Weak},
    sync::Arc,
};

use crate::{
    utils::op_has_trait, AllocId, ContextRef, ContextWRef, OpRef, Terminator, Type, Validate,
    ValidateErr, Value,
};

pub type RegionRef = Rc<Region>;
pub type RegionWRef = Weak<Region>;
pub type BlockRef = Rc<Block>;
pub type BlockWRef = Weak<Block>;

#[derive(Debug, Clone)]
pub struct BlockArg {
    parent: BlockWRef,
    #[allow(dead_code)]
    index: usize,
    ty: Type,
}

impl BlockArg {
    pub fn get_type(&self) -> Type {
        self.ty.clone()
    }

    pub fn get_block(&self) -> Option<BlockRef> {
        self.parent.upgrade()
    }
}

impl PartialEq for BlockArg {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

#[derive(Debug)]
struct BlockImpl {
    name: String,
    parent_region: RegionWRef,
    operations: Vec<AllocId>,
    args: Vec<Value>,
}

impl BlockImpl {
    fn new(name: String, parent_region: RegionWRef) -> Self {
        Self {
            name,
            parent_region,
            operations: vec![],
            args: vec![],
        }
    }

    fn push(&mut self, op: &OpRef) {
        self.operations.push(op.borrow().get_alloc_id());

        op.borrow_mut()
            .set_parent_region(self.parent_region.clone());
    }

    fn insert(&mut self, index: usize, op: &OpRef) {
        self.operations.insert(index, op.borrow().get_alloc_id());

        op.borrow_mut()
            .set_parent_region(self.parent_region.clone());
    }

    fn erase(&mut self, op: &OpRef) {
        let index = self
            .operations
            .iter()
            .position(|x| *x == op.borrow().get_alloc_id())
            .unwrap();
        self.operations.remove(index);
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

    fn last(&self) -> Option<OpRef> {
        let context = self.get_context();
        self.operations.last().map(|id| context.get_op(*id))?
    }

    fn add_argument(&mut self, ty: Type, name: &str, this: BlockWRef) {
        let index = self.args.len();

        let context = Arc::downgrade(&ty.get_context().unwrap());

        let block_arg = BlockArg {
            parent: this,
            index,
            ty,
        };

        self.args
            .push(Value::from_block_arg(context, name, block_arg));
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn find(&self, op_id: AllocId) -> Option<usize> {
        (0..self.operations.len()).find(|&idx| self.operations[idx] == op_id)
    }
}

#[derive(Debug)]
pub struct Block(RefCell<BlockImpl>);

pub struct BlockIter {
    context: ContextRef,
    data: Vec<AllocId>,
    index: usize,
}

impl Iterator for BlockIter {
    type Item = OpRef;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.data.get(self.index);
        self.index += 1;
        id.map(|id| self.context.get_op(*id))?
    }
}

pub struct BlockArgIter {
    data: Vec<Value>,
    index: usize,
}

impl Iterator for BlockArgIter {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        let data = self.data.get(self.index);
        self.index += 1;
        data.cloned()
    }
}

impl Block {
    pub fn empty(parent: &RegionRef) -> BlockRef {
        Rc::new(Block(RefCell::new(BlockImpl::new(
            "entry".to_string(),
            Rc::downgrade(parent),
        ))))
    }

    pub fn with_arguments<T: AsRef<str>>(
        name: &str,
        parent: &RegionRef,
        arg_types: &[Type],
        arg_names: &[T],
    ) -> BlockRef {
        let block = Rc::new(Block(RefCell::new(BlockImpl::new(
            name.to_string(),
            Rc::downgrade(parent),
        ))));

        block.clone().add_arguments(zip(arg_names, arg_types));

        block
    }

    pub fn push(&self, op: &OpRef) {
        self.0.borrow_mut().push(op);
    }

    pub fn insert(&self, index: usize, op: &OpRef) {
        self.0.borrow_mut().insert(index, op);
    }

    pub fn erase(&self, op: &OpRef) {
        self.0.borrow_mut().erase(op);
    }

    pub fn get_parent_region(&self) -> RegionRef {
        self.0.borrow().get_parent_region()
    }

    pub fn first(&self) -> Option<OpRef> {
        self.0.borrow().first()
    }

    pub fn last(&self) -> Option<OpRef> {
        self.0.borrow().last()
    }

    pub fn get_context(&self) -> ContextRef {
        self.0.borrow().get_context()
    }

    pub fn iter(&self) -> BlockIter {
        BlockIter {
            context: self.get_context(),
            data: self.0.borrow().operations.clone(),
            index: 0,
        }
    }

    pub fn get_args(&self) -> BlockArgIter {
        BlockArgIter {
            data: self.0.borrow().args.clone(),
            index: 0,
        }
    }

    pub fn add_arguments<'s, S: AsRef<str>, T: IntoIterator<Item = (S, &'s Type)>>(
        self: Rc<Self>,
        args: T,
    ) {
        let this = Rc::downgrade(&self);
        for (name, ty) in args {
            self.0
                .borrow_mut()
                .add_argument(ty.clone(), name.as_ref(), this.clone());
        }
    }

    pub fn find(&self, op_id: AllocId) -> Option<usize> {
        self.0.borrow().find(op_id)
    }

    pub fn get_name(&self) -> String {
        self.0.borrow().get_name()
    }
}

impl Validate for Block {
    fn validate(&self) -> Result<(), ValidateErr> {
        let region = self.get_parent_region();
        let self_ref = if let Some(blk) = region.get_block_by_name(&self.get_name()) {
            blk
        } else {
            return Err(ValidateErr::BlockNotRegisteredWithRegion(self.get_name()));
        };

        if let Some(op) = self.last() {
            if !op_has_trait::<dyn Terminator>(op) {
                return Err(ValidateErr::BlockMissingTerminator(self_ref));
            }
        } else {
            return Err(ValidateErr::BlockMissingTerminator(self_ref));
        }

        for op in self.iter() {
            op.borrow().validate()?
        }

        Ok(())
    }
}

pub struct RegionIter {
    data: Vec<BlockRef>,
    index: usize,
}

impl Iterator for RegionIter {
    type Item = BlockRef;

    fn next(&mut self) -> Option<Self::Item> {
        let data = self.data.get(self.index);
        self.index += 1;
        data.cloned()
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Region(RefCell<RegionImpl>);

impl Region {
    pub fn empty(context: &ContextRef) -> RegionRef {
        Rc::new(Region(RefCell::new(RegionImpl::new(Arc::downgrade(
            context,
        )))))
    }

    pub fn with_single_block(context: &ContextRef) -> RegionRef {
        let region = Region::empty(context);
        let block = Block::empty(&region);
        region.add_block(block);

        region
    }

    pub fn get_context(&self) -> ContextRef {
        self.0.borrow().get_context()
    }

    pub fn add_block(&self, block: BlockRef) {
        self.0.borrow_mut().blocks.push(block);
    }

    pub fn get_parent_op(&self) -> OpRef {
        let context = self.0.borrow().context.upgrade().unwrap();
        context.get_op(self.0.borrow().parent_op).unwrap()
    }

    pub fn first(&self) -> Option<BlockRef> {
        self.0.borrow().blocks.first().cloned()
    }

    pub fn iter(&self) -> RegionIter {
        RegionIter {
            data: self.0.borrow().blocks.clone(),
            index: 0,
        }
    }

    pub fn get_block_by_name(&self, name: &str) -> Option<BlockRef> {
        self.iter().find(|blk| blk.get_name() == name)
    }

    pub fn find_op_block(&self, op: &OpRef) -> Option<BlockRef> {
        self.iter()
            .find(|blk| blk.find(op.borrow().get_alloc_id()).is_some())
    }
}

impl Validate for Region {
    fn validate(&self) -> Result<(), ValidateErr> {
        for blk in self.iter() {
            blk.validate()?
        }
        Ok(())
    }
}
