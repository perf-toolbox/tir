use std::cell::{Ref, RefCell};
use std::fmt::Debug;
use std::rc::Rc;
use tir_backend::BinaryEmittable;
use tir_core::utils::{trait_id, TraitId};
use tir_core::{Op, Operand, Operation, OperationImpl};
use tir_macros::operation;

use crate::DIALECT_NAME;

pub struct RTypeInstr {
    instr: u32,
}

impl RTypeInstr {
    pub fn from_bytes(bytes: &[u8; 4]) -> Self {
        RTypeInstr {
            instr: u32::from_le_bytes(*bytes),
        }
    }

    pub fn builder() -> RTypeBuilder {
        RTypeBuilder::default()
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        self.instr.to_le_bytes()
    }

    pub fn opcode(&self) -> u8 {
        (self.instr & 0b1111111) as u8
    }

    pub fn rd(&self) -> u8 {
        ((self.instr & (0b11111 << 7)) >> 7) as u8
    }

    pub fn funct3(&self) -> u8 {
        ((self.instr & (0b111 << 12)) >> 12) as u8
    }

    pub fn rs1(&self) -> u8 {
        ((self.instr & (0b11111 << 15)) >> 15) as u8
    }

    pub fn rs2(&self) -> u8 {
        ((self.instr & (0b11111 << 20)) >> 20) as u8
    }

    pub fn funct7(&self) -> u8 {
        ((self.instr & (0b1111111 << 25)) >> 25) as u8
    }
}

#[derive(Default)]
pub struct RTypeBuilder {
    instr: u32,
}

impl RTypeBuilder {
    pub fn opcode(mut self, opcode: u8) -> Self {
        assert!(opcode <= 0b1111111);
        self.instr += opcode as u32;
        self
    }

    pub fn rd(mut self, rd: u8) -> Self {
        assert!(rd <= 0b11111);
        self.instr += (rd as u32) << 7;
        self
    }

    pub fn funct3(mut self, funct3: u8) -> Self {
        assert!(funct3 <= 0b111);
        self.instr += (funct3 as u32) << 12;
        self
    }

    pub fn rs1(mut self, rs1: u8) -> Self {
        assert!(rs1 <= 0b11111);
        self.instr += (rs1 as u32) << 15;
        self
    }

    pub fn rs2(mut self, rs2: u8) -> Self {
        assert!(rs2 <= 0b11111);
        self.instr += (rs2 as u32) << 20;
        self
    }

    pub fn funct7(mut self, funct7: u8) -> Self {
        assert!(funct7 <= 0b1111111);
        self.instr += (funct7 as u32) << 25;
        self
    }

    pub fn build(self) -> RTypeInstr {
        RTypeInstr { instr: self.instr }
    }
}

impl Debug for RTypeInstr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = format!("{:#032b}: opcode = {:#07b}, rd = {:#05b}, funct3 = {:#03b}, rs1 = {:#05b}, rs2 = {:#05b}, funct7 = {:#07b}", self.instr, self.opcode(), self.rd(), self.funct3(), self.rs1(), self.rs2(), self.funct7());
        f.write_str(&string)
    }
}

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
                stream: &mut Box<dyn tir_backend::BinaryStream>,
            ) -> tir_core::Result<()> {
                let instr = RTypeInstr::builder()
                    .opcode(0b010011)
                    .rd(0b000)
                    .funct3($funct3)
                    .rs1(0b000000)
                    .rs2(0b000000)
                    .funct7($funct7)
                    .build();
                stream.write(&instr.to_bytes());
                Ok(())
            }
        }
        )*
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
    use crate::RTypeInstr;

    #[test]
    fn test_rtype() {
        let word: u32 = 7537331;
        let bytes = word.to_le_bytes();

        let instr = RTypeInstr::from_bytes(&bytes);

        println!("{:?}", instr);
        assert_eq!(instr.opcode(), 0b0110011_u8);
        assert_eq!(instr.rd(), 5);
        assert_eq!(instr.funct3(), 0);
        assert_eq!(instr.rs1(), 6);
        assert_eq!(instr.rs2(), 7);
        assert_eq!(instr.funct7(), 0);
    }

    #[test]
    fn test_rtype_builder() {
        let instr = RTypeInstr::builder()
            .opcode(0b11111)
            .funct3(0b111)
            .rd(0b11011)
            .rs1(0b00001)
            .rs2(0b11000)
            .funct7(0b1100000)
            .build();

        assert_eq!(instr.opcode(), 0b11111);
        assert_eq!(instr.funct3(), 0b111);
        assert_eq!(instr.rd(), 0b11011);
        assert_eq!(instr.rs1(), 0b00001);
        assert_eq!(instr.rs2(), 0b11000);
        assert_eq!(instr.funct7(), 0b1100000);
    }
}
