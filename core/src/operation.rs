use crate::{Attr, Context, Dialect};
use std::collections::HashMap;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

#[derive(Debug)]
pub enum Operand {
    Value(Value),
    BlockArgument,
    Block(Block),
    Register(i32),
}

#[derive(Debug)]
pub struct Value {
    operation: Rc<RefCell<Operation>>,
    result_id: u32,
}

#[derive(Debug)]
pub struct Block {
    parent: Weak<RefCell<Region>>,
    pub operations: Vec<Rc<RefCell<Operation>>>,
}

impl Block {
    pub fn new(parent: Weak<RefCell<Region>>) -> Rc<RefCell<Block>> {
        Rc::new(RefCell::new(Block {
            parent,
            operations: vec![],
        }))
    }

    pub fn add_operation(&mut self, operation: Rc<RefCell<Operation>>) {
        self.operations.push(operation);
    }

    pub fn get_operations(&self) -> &[Rc<RefCell<Operation>>] {
        &self.operations
    }

    pub fn get_parent(&self) -> Weak<RefCell<Region>> {
        self.parent.clone()
    }
}

#[derive(Debug)]
pub struct Region {
    context: Rc<RefCell<Context>>,
    pub blocks: Vec<Rc<RefCell<Block>>>,
}

impl Region {
    pub fn new(context: Rc<RefCell<Context>>) -> Rc<RefCell<Region>> {
        Rc::new(RefCell::new(Region {
            context: context.clone(),
            blocks: vec![],
        }))
    }

    pub fn get_context(&self) -> Rc<RefCell<Context>> {
        self.context.clone()
    }

    pub fn emplace_block(&mut self, parent: Weak<RefCell<Region>>) -> Rc<RefCell<Block>> {
        let block = Block::new(parent);
        self.blocks.push(block.clone());
        block
    }

    pub fn get_blocks(&self) -> &[Rc<RefCell<Block>>] {
        &self.blocks
    }
}

#[derive(Debug)]
pub struct Operation {
    context: Rc<RefCell<Context>>,
    dialect_id: u32,
    operation_id: u32,
    operation_name: &'static str,
    operands: Vec<Operand>,
    attrs: HashMap<String, Attr>,
    regions: Vec<Rc<RefCell<Region>>>,
}

impl Operation {
    pub fn new(
        context: Rc<RefCell<Context>>,
        dialect: Rc<RefCell<Dialect>>,
        operation_name: &'static str,
    ) -> Rc<RefCell<Operation>> {
        let dialect_id = dialect.borrow().get_id();
        let operation_id = dialect.borrow().get_operation_id(operation_name);
        Rc::new(RefCell::new(Operation {
            context,
            dialect_id,
            operation_id,
            operation_name,
            operands: vec![],
            attrs: HashMap::new(),
            regions: vec![],
        }))
    }
    pub fn get_context(&self) -> Rc<RefCell<Context>> {
        self.context.clone()
    }

    pub fn get_dialect_id(&self) -> u32 {
        self.dialect_id
    }

    pub fn get_operation_id(&self) -> u32 {
        self.operation_id
    }

    pub fn get_operation_name(&self) -> &'static str {
        self.operation_name
    }

    pub fn emplace_region(&mut self) -> Rc<RefCell<Region>> {
        let region = Region::new(self.get_context());
        self.regions.push(region.clone());
        region
    }

    pub fn get_regions(&self) -> &[Rc<RefCell<Region>>] {
        &self.regions
    }

    pub fn add_attr(&mut self, name: String, attr: Attr) {
        self.attrs.insert(name, attr);
    }

    pub fn get_attrs(&self) -> &HashMap<String, Attr> {
        &self.attrs
    }
}

pub trait Op: Into<Rc<RefCell<Operation>>> + Sized {
    fn get_operation_name() -> &'static str;
}
