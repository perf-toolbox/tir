use std::cell::RefCell;
use std::rc::Rc;
use tir_core::builtin::ModuleOp;
use tir_core::ContextRef;

use crate::RegFile;

pub struct Simulator {
    module: Rc<RefCell<ModuleOp>>,
}

impl Simulator {
    pub fn new(module: Rc<RefCell<ModuleOp>>) -> Self {
        Simulator { module }
    }

    pub fn run(&self, reg_file: &Rc<RefCell<dyn RegFile>>) {
        todo!()
    }
}
