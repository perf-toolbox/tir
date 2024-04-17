use std::cell::Ref;
use std::cell::RefCell;
use std::rc::Rc;

use crate::DIALECT_NAME;
use tir_backend::BinaryEmittable;
use tir_core::utils::{trait_id, TraitId};
use tir_core::{Op, Operand, Operation, OperationImpl};
use tir_macros::operation;

macro_rules! alu_op_base {
    ($struct_name:ident, $op_name:literal) => {
        #[operation(name = $op_name, traits(BinaryEmittable))]
        pub struct $struct_name {
            #[cfg(operand = true)]
            rs1: Register,
            #[cfg(operand = true)]
            rs2: Register,
            #[cfg(operand = true)]
            rd: Register,
        }
    };
}

macro_rules! alu_op {
    ($struct_name:ident, $op_name:literal) => {
        alu_op_base!($struct_name, $op_name);
    };
}

alu_op!(AddOp, "add");
alu_op!(SubOp, "sub");
alu_op!(SllOp, "sll");
alu_op!(SltOp, "slt");
alu_op!(SltuOp, "sltu");
alu_op!(XorOp, "xor");
alu_op!(SrlOp, "srl");
alu_op!(SraOp, "sra");
alu_op!(OrOp, "or");
alu_op!(AndOp, "and");

impl BinaryEmittable for AddOp {
    fn encode(
        &self,
        target_opts: &tir_backend::TargetOptions,
        stream: &mut Box<dyn tir_backend::BinaryStream>,
    ) -> tir_core::Result<()> {
        Ok(())
    }

    fn try_decode(data: &[u8]) -> tir_core::Result<Operation> {
        todo!()
    }
}
