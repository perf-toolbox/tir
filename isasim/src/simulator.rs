use std::cell::RefCell;
use std::rc::Rc;
use tir_core::builtin::ModuleOp;
use tir_core::OpRef;
use tir_macros::match_op;

use crate::{RegFile, Value};

pub struct Simulator {
    module: Rc<RefCell<ModuleOp>>,
}

macro_rules! exec_alu {
    ($name:ident, $op_ty:ty, $op:tt) => {
        fn $name(op: &Rc<RefCell<$op_ty>>, reg_file: &Rc<RefCell<dyn RegFile>>) {
            let rs1: String = op
                .borrow()
                .get_rs1_attr()
                .clone()
                .try_into()
                .expect("reg name is a String attr");
            let rs2: String = op
                .borrow()
                .get_rs2_attr()
                .clone()
                .try_into()
                .expect("reg name is a String attr");
            let rd: String = op
                .borrow()
                .get_rd_attr()
                .clone()
                .try_into()
                .expect("reg name is a String attr");

            let a = reg_file.borrow().read_register(&rs1).get_lower();
            let b = reg_file.borrow().read_register(&rs2).get_lower();

            let c = a $op b;
            let c = Value::from(c);

            reg_file.borrow_mut().write_register(&rd, &c);
        }

    };
}

exec_alu!(exec_add, tir_backend::isema::AddOp, +);
exec_alu!(exec_sub, tir_backend::isema::SubOp, -);
exec_alu!(exec_and, tir_backend::isema::AndOp, &);
exec_alu!(exec_or, tir_backend::isema::OrOp, |);
exec_alu!(exec_xor, tir_backend::isema::XorOp, ^);
exec_alu!(exec_sll, tir_backend::isema::SllOp, <<);
exec_alu!(exec_srl, tir_backend::isema::SrlOp, >>);

fn execute_op(op: &OpRef, reg_file: &Rc<RefCell<dyn RegFile>>) {
    use tir_backend::isema::*;

    let op = op.clone();
    match_op!(op {
        AddOp => |add| exec_add(&add, reg_file),
        SubOp => |sub| exec_sub(&sub, reg_file),
        AndOp => |and| exec_and(&and, reg_file),
        OrOp => |or| exec_or(&or, reg_file),
        XorOp => |xor| exec_xor(&xor, reg_file),
        SllOp => |sll| exec_sll(&sll, reg_file),
        SrlOp => |srl| exec_srl(&srl, reg_file),
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
