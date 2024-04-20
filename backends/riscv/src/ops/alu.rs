use crate::utils::RTypeInstr;
use crate::{disassemble_gpr, encode_gpr};
use std::cell::{Ref, RefCell};
use std::rc::Rc;
use tir_backend::BinaryEmittable;
use tir_core::utils::{trait_id, TraitId};
use tir_core::{Context, Op, Operand, Operation, OperationImpl};
use tir_macros::operation;

use crate::DIALECT_NAME;

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

        impl $struct_name {
            pub fn new(
                context: Rc<RefCell<Context>>,
                rs1: Operand,
                rs2: Operand,
                rd: Operand,
            ) -> Self {
                let dialect = context.borrow().get_dialect_by_name(DIALECT_NAME).unwrap();
                let mut operation =
                    Operation::new(context.clone(), dialect, $struct_name::get_operation_name());
                operation.add_operand(rs1);
                operation.add_operand(rs2);
                operation.add_operand(rd);

                Self {
                    operation: operation.get_impl(),
                }
            }
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
                stream: &mut Box<dyn tir_backend::BinaryStream>,
            ) -> tir_core::Result<()> {
                let instr = RTypeInstr::builder()
                    .opcode(0b010011)
                    .rd(encode_gpr(&self.get_rd())?)
                    .funct3($funct3)
                    .rs1(encode_gpr(&self.get_rs1())?)
                    .rs2(encode_gpr(&self.get_rs2())?)
                    .funct7($funct7)
                    .build();
                stream.write(&instr.to_bytes());
                Ok(())
            }
        }
        )*

        pub fn disassemble_alu_instr(context: &Rc<RefCell<Context>>, stream: &[u8]) -> Option<Operation> {
            if stream.len() < 4 {
                return None;
            }

            let instr = RTypeInstr::from_bytes(&stream[0..4].try_into().unwrap());
            if instr.opcode() != 0b011011 {
                return None;
            }

            let rd = disassemble_gpr(instr.rd())?;
            let rs1 = disassemble_gpr(instr.rs1())?;
            let rs2 = disassemble_gpr(instr.rs2())?;

            match (instr.funct3(), instr.funct7()) {
                $(
                ($funct3, $funct7) => {
                    let op = $struct_name::new(context.clone(), rs1, rs2, rd);
                    Some(op.into())
                },
                )*
                _ => None,
            }
        }
    };
}

// FIXME: all popular CPUs (x86, arm, risc-v) use little-endian. What happens if this code is
// compiled on a big-endian host?
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
