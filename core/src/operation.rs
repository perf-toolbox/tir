use crate::{Attr, ContextRef};
use std::collections::HashMap;

use std::{
    any::Any,
    cell::RefCell,
    rc::{Rc, Weak},
};

pub type BlockRef = Rc<RefCell<Block>>;
pub type RegionRef = Rc<RefCell<Region>>;
pub type RegionWeakRef = Weak<RefCell<Region>>;

pub enum Operand {
    Value(Value),
    BlockArgument,
    Block(Block),
    Register(u32),
    RegisterClass(u32),
}

pub struct Value {
    _operation: Operation,
    _result_id: u32,
}

pub struct Block {
    parent: RegionWeakRef,
    pub operations: Vec<Operation>,
}

impl Block {
    pub fn new(parent: RegionWeakRef) -> Rc<RefCell<Block>> {
        Rc::new(RefCell::new(Block {
            parent,
            operations: vec![],
        }))
    }

    pub fn add_operation(&mut self, operation: Operation) {
        self.operations.push(operation);
    }

    pub fn get_operations(&self) -> &[Operation] {
        &self.operations
    }

    pub fn get_parent(&self) -> RegionWeakRef {
        self.parent.clone()
    }
}

pub struct Region {
    context: ContextRef,
    pub blocks: Vec<BlockRef>,
}

impl Region {
    pub fn new(context: ContextRef) -> RegionRef {
        Rc::new(RefCell::new(Region {
            context: context.clone(),
            blocks: vec![],
        }))
    }

    pub fn get_context(&self) -> ContextRef {
        self.context.clone()
    }

    pub fn emplace_block(&mut self, parent: RegionWeakRef) -> BlockRef {
        let block = Block::new(parent);
        self.blocks.push(block.clone());
        block
    }

    pub fn get_blocks(&self) -> &[BlockRef] {
        &self.blocks
    }
}

pub struct OperationImpl {
    pub context: ContextRef,
    pub dialect_id: u32,
    pub operation_id: u32,
    pub operands: Vec<Operand>,
    pub attrs: HashMap<String, Attr>,
    pub regions: Vec<Rc<RefCell<Region>>>,
}

impl OperationImpl {
    pub fn get_regions(&self) -> &[RegionRef] {
        &self.regions
    }
}

pub type Operation = Rc<RefCell<dyn Op>>;

pub trait Op: Any {
    fn get_operation_name() -> &'static str
    where
        Self: Sized;

    fn has_trait<T: ?Sized + 'static>() -> bool
    where
        Self: Sized;

    fn get_context(&self) -> ContextRef;
    fn get_dialect_id(&self) -> u32;
    fn emplace_region(&mut self) -> RegionRef;
    fn get_regions(&self) -> &[RegionRef];
    fn add_attr(&mut self, name: String, attr: Attr);
    fn get_attrs(&self) -> &HashMap<String, Attr>;
    fn add_operand(&mut self, operand: Operand);
}
