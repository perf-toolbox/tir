use crate::utils::RTypeInstr;
use crate::{disassemble_gpr, encode_gpr};
use std::cell::{Ref, RefCell};
use std::rc::Rc;
use tir_backend::BinaryEmittable;
use tir_core::utils::{trait_id, TraitId};
use tir_core::{Context, Op, Operand, Operation, OperationImpl};
use tir_macros::operation;

use crate::DIALECT_NAME;

const ALU_OPCODE: u8 = 0b110011;

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
                    .opcode(ALU_OPCODE)
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
            if instr.opcode() != ALU_OPCODE {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disassemble_alu_instr;
    use tir_core::Context;

    #[test]
    fn test_disassembler() {
        // add x28, x6, x7
        // sub x28, x6, x7
        // sll x28, x6, x7
        // slt x28, x6, x7
        // sltu x28, x6, x7
        // srl x28, x6, x7
        // sra x28, x6, x7
        // or x28, x6, x7
        // and x28, x6, x7
        let instructions = vec![
            0x00730e33 as u32,
            0x40730e33,
            0x00731e33,
            0x00732e33,
            0x00733e33,
            0x00735e33,
            0x40735e33,
            0x00736e33,
            0x00737e33,
        ];

        let context = Context::new();
        context.borrow_mut().add_dialect(crate::create_dialect());

        let mut ops = vec![];

        for instr in instructions {
            if let Some(op) = disassemble_alu_instr(&context, &instr.to_le_bytes()) {
                ops.push(op);
            }
        }

        assert_eq!(ops.len(), 9);
        assert!(AddOp::try_from(ops[0].clone()).is_ok());
        assert!(SubOp::try_from(ops[1].clone()).is_ok());
        assert!(SllOp::try_from(ops[2].clone()).is_ok());
        assert!(SltOp::try_from(ops[3].clone()).is_ok());
        assert!(SltuOp::try_from(ops[4].clone()).is_ok());
        assert!(SrlOp::try_from(ops[5].clone()).is_ok());
        assert!(SraOp::try_from(ops[6].clone()).is_ok());
        assert!(OrOp::try_from(ops[7].clone()).is_ok());
        assert!(AndOp::try_from(ops[8].clone()).is_ok());
    }

    #[test]
    fn test_disassembler_negative() {
        // _boot:
        //   addi x28, x6, 1000
        //   jal _boot
        // some bogus instr
        let instructions = vec![0x3e830e13 as u32, 0xffdff0ef, 0x7fffff3];

        let context = Context::new();
        context.borrow_mut().add_dialect(crate::create_dialect());

        let mut ops = vec![];

        for instr in instructions {
            if let Some(op) = disassemble_alu_instr(&context, &instr.to_le_bytes()) {
                ops.push(op);
            }
        }

        assert_eq!(ops.len(), 0);
    }
}
