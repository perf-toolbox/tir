use crate::{Attr, Context, Dialect};
use std::collections::HashMap;
use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

pub type BlockRef = Rc<RefCell<Block>>;

#[derive(Debug)]
pub enum Operand {
    Value(Value),
    BlockArgument,
    Block(Block),
    Register(u32),
    RegisterClass(u32),
}

#[derive(Debug)]
pub struct Value {
    _operation: Rc<RefCell<Operation>>,
    _result_id: u32,
}

#[derive(Debug)]
pub struct Block {
    parent: Weak<RefCell<Region>>,
    pub operations: Vec<Operation>,
}

impl Block {
    pub fn new(parent: Weak<RefCell<Region>>) -> Rc<RefCell<Block>> {
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
pub struct OperationImpl {
    pub context: Rc<RefCell<Context>>,
    pub dialect_id: u32,
    pub operation_id: u32,
    pub operation_name: &'static str,
    pub operands: Vec<Operand>,
    pub attrs: HashMap<String, Attr>,
    pub regions: Vec<Rc<RefCell<Region>>>,
}

impl OperationImpl {
    pub fn get_regions(&self) -> &[Rc<RefCell<Region>>] {
        &self.regions
    }
}

#[derive(Debug, Clone)]
pub struct Operation {
    r#impl: Rc<RefCell<OperationImpl>>,
}

impl Operation {
    pub fn new(
        context: Rc<RefCell<Context>>,
        dialect: Rc<RefCell<Dialect>>,
        operation_name: &'static str,
    ) -> Operation {
        let dialect_id = dialect.borrow().get_id();
        let operation_id = dialect.borrow().get_operation_id(operation_name);
        let r#impl = Rc::new(RefCell::new(OperationImpl {
            context,
            dialect_id,
            operation_id,
            operation_name,
            operands: vec![],
            attrs: HashMap::new(),
            regions: vec![],
        }));
        Operation { r#impl }
    }

    pub fn from(r#impl: Rc<RefCell<OperationImpl>>) -> Self {
        Operation { r#impl }
    }

    pub fn get_context(&self) -> Rc<RefCell<Context>> {
        self.r#impl.borrow().context.clone()
    }

    pub fn get_dialect_id(&self) -> u32 {
        self.r#impl.borrow().dialect_id
    }

    pub fn get_operation_id(&self) -> u32 {
        self.r#impl.borrow().operation_id
    }

    pub fn get_operation_name(&self) -> &'static str {
        self.r#impl.borrow().operation_name
    }

    pub fn emplace_region(&mut self) -> Rc<RefCell<Region>> {
        let region = Region::new(self.get_context());
        self.r#impl.borrow_mut().regions.push(region.clone());
        region
    }

    pub fn get_regions(&self) -> Ref<'_, [Rc<RefCell<Region>>]> {
        Ref::map(self.r#impl.borrow(), |r#impl| r#impl.get_regions())
    }

    pub fn add_attr(&mut self, name: String, attr: Attr) {
        self.r#impl.borrow_mut().attrs.insert(name, attr);
    }

    pub fn get_attrs(&self) -> Ref<'_, HashMap<String, Attr>> {
        Ref::map(self.r#impl.borrow(), |r#impl| &r#impl.attrs)
    }

    pub fn get_impl(&self) -> Rc<RefCell<OperationImpl>> {
        self.r#impl.clone()
    }

    pub fn add_operand(&mut self, operand: Operand) {
        self.r#impl.borrow_mut().operands.push(operand);
    }
}

pub trait Op {
    fn get_operation_name() -> &'static str
    where
        Self: Sized;

    fn has_trait<T: ?Sized + 'static>() -> bool
    where
        Self: Sized;
}
