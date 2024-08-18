use std::rc::Rc;
use std::{cell::RefCell, ops::BitAnd};
use tir_core::builtin::ModuleOp;
use tir_core::{OpRef, StdoutPrinter};
use tir_macros::match_op;

use crate::{MemoryMap, RegFile, Value};

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

fn execute_load(
    op: &Rc<RefCell<tir_backend::isema::LoadOp>>,
    reg_file: &Rc<RefCell<dyn RegFile>>,
    mem: &Rc<RefCell<MemoryMap>>,
) {
    let base_reg: String = op
        .borrow()
        .get_base_addr_attr()
        .clone()
        .try_into()
        .expect("reg name is a String attr");

    let base_addr = reg_file.borrow().read_register(&base_reg).get_lower() as u64;

    let offset: i16 = op.borrow().get_offset_attr().try_into().expect("");

    let addr = (base_addr as i64 + offset as i64) as u64;

    let width: i32 = op.borrow().get_width_attr().try_into().expect("");

    let mut data = mem.borrow().load(addr, (width / 8) as u8).expect("");

    let sign_extend: bool = op.borrow().get_sign_extend_attr().try_into().expect("");

    let extent: u8 = if sign_extend && data.last().unwrap().bitand(1 << 7) != 0 {
        255
    } else {
        0
    };
    for _ in 0..(reg_file.borrow().base_width() as usize - data.len()) {
        data.push(extent);
    }

    let reg_value: crate::Value = data.try_into().expect("");

    let dst: String = op.borrow().get_dst_attr().clone().try_into().expect("");

    reg_file.borrow_mut().write_register(&dst, &reg_value);
}

fn execute_store(
    op: &Rc<RefCell<tir_backend::isema::StoreOp>>,
    reg_file: &Rc<RefCell<dyn RegFile>>,
    mem: &Rc<RefCell<MemoryMap>>,
) {
    let base_reg: String = op
        .borrow()
        .get_base_addr_attr()
        .clone()
        .try_into()
        .expect("reg name is a String attr");

    let base_addr = reg_file.borrow().read_register(&base_reg).get_lower() as u64;
    let offset: i16 = op.borrow().get_offset_attr().try_into().expect("");

    let addr = (base_addr as i64 + offset as i64) as u64;
    let width: i32 = op.borrow().get_width_attr().try_into().expect("");

    let src: String = op.borrow().get_src_attr().clone().try_into().expect("");
    let value = reg_file
        .borrow()
        .read_register(&src)
        .raw_bytes((width / 8) as usize)
        .expect("");

    mem.borrow_mut().store(addr, &value).expect("");
}

fn execute_op(op: &OpRef, reg_file: &Rc<RefCell<dyn RegFile>>, mem: &Rc<RefCell<MemoryMap>>) {
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
        LoadOp => |load| execute_load(&load, reg_file, mem),
        StoreOp => |store| execute_store(&store, reg_file, mem),
        _ => || {
            let mut printer = StdoutPrinter::new();
            op.borrow().print(&mut printer);
            println!("FAIL")
        },

    });
}

impl Simulator {
    pub fn new(module: Rc<RefCell<ModuleOp>>) -> Self {
        Simulator { module }
    }

    pub fn run(&self, reg_file: &Rc<RefCell<dyn RegFile>>, mem: &Rc<RefCell<MemoryMap>>) {
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
                        execute_op(&op, reg_file, mem);
                    }
                }
            }
        }
    }
}
