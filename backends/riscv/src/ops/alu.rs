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

macro_rules! alu_ops {
    // R-format ALU operations
    ($($struct_name:ident => { name = $op_name:literal, funct7 = $funct7:literal, funct3 = $funct3:literal })*) => {
        $(
        alu_op_base!($struct_name, $op_name);
        )*

        $(
        impl BinaryEmittable for $struct_name {
            fn encode(
                &self,
                _target_opts: &tir_backend::TargetOptions,
                _stream: &mut Box<dyn tir_backend::BinaryStream>,
            ) -> tir_core::Result<()> {
                Ok(())
            }

            fn try_decode(_data: &[u8]) -> tir_core::Result<Operation> {
                todo!()
            }
        }
        )*
    };
}

alu_ops! {
    AddOp => { name = "add", funct7 = 0b0000000, funct3 = 0b000 }
    SubOp => { name = "sub", funct7 = 0b0100000, funct3 = 0b000 }
    SllOp => { name = "sll", funct7 = 0b0000000, funct3 = 0b001 }
    SltOp => { name = "slt", funct7 = 0b0000000, funct3 = 0b010 }
    SltuOp => { name = "sltu", funct7 = 0b0000000, funct3 = 0b011 }
    SrlOp => { name = "srl", funct7 = 0b0000000, funct3 = 0b101 }
    SraOp => { name = "sra", funct7 = 0b0100000, funct3 = 0b101 }
    OrOp => { name = "or", funct7 = 0b0000000, funct3 = 0b110 }
    AndOp => { name = "and", funct7 = 0b0000000, funct3 = 0b111 }
}

