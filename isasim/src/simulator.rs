use std::cell::RefCell;
use std::rc::Rc;
use tir_core::builtin::ModuleOp;
use tir_core::OpRef;
use tir_macros::match_op;

use crate::{RegFile, Value};

pub struct Simulator {
    module: Rc<RefCell<ModuleOp>>,
}

fn exec_add(add: &Rc<RefCell<tir_backend::isema::AddOp>>, reg_file: &Rc<RefCell<dyn RegFile>>) {
    let rs1: String = add
        .borrow()
        .get_rs1_attr()
        .clone()
        .try_into()
        .expect("reg name is a String attr");
    let rs2: String = add
        .borrow()
        .get_rs2_attr()
        .clone()
        .try_into()
        .expect("reg name is a String attr");
    let rd: String = add
        .borrow()
        .get_rd_attr()
        .clone()
        .try_into()
        .expect("reg name is a String attr");

    let a = reg_file.borrow().read_register(&rs1).get_lower();
    let b = reg_file.borrow().read_register(&rs2).get_lower();

    let c = a + b;
    let c = Value::from(c);

    reg_file.borrow_mut().write_register(&rd, &c);
}

fn execute_op(op: &OpRef, reg_file: &Rc<RefCell<dyn RegFile>>) {
    use tir_backend::isema::*;

    let op = op.clone();
    match_op!(op {
        AddOp => |add| exec_add(&add, reg_file),
        SubOp => |_| println!("Sub!"),
        _ => || println!("FAIL"),
    });
}

impl Simulator {
    pub fn new(module: Rc<RefCell<ModuleOp>>) -> Self {
        Simulator { module }
    }

    pub fn run(&self, reg_file: &Rc<RefCell<dyn RegFile>>) {
        let iter = self.module.borrow().get_body().iter();
        for instr in iter {
            if let Some(section) =
                tir_core::utils::op_cast::<tir_backend::target::SectionOp>(instr.clone())
            {
                let attr = section.borrow().get_name_attr().clone();
                let name: String = attr.try_into().unwrap();
                if name != ".text" {
                    continue;
                }

                let section_iter = section.borrow().get_body_region().iter();
                for block in section_iter {
                    let block_iter = block.iter();

                    for op in block_iter {
                        execute_op(&op, reg_file);
                    }
                }
            }
        }
    }
}
