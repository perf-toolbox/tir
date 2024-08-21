use crate::utils::RTypeInstr;
use crate::utils::ITypeInstr;
use crate::{assemble_reg, disassemble_gpr};
use crate::{register_parser, Register};
use tir_backend::isema;
use tir_backend::isema::WithISema;
use tir_backend::AsmToken;
use tir_backend::BinaryEmittable;
use tir_backend::ISAParser;
use tir_backend::TokenStream;
use tir_core::parser::{AsmPResult, Parsable};
use tir_core::OpAssembly;
use tir_core::*;
use tir_macros::{lowercase, uppercase};
use tir_macros::{Op, OpAssembly, OpValidator};
use winnow::combinator::{preceded, separated, separated_pair};
use winnow::token::one_of;
use winnow::Parser;

use crate::DIALECT_NAME;

const ALU_OPCODE: u8 = 0b110011;
const ALU_IMM_OPCODE: u8 = 0b0010011;

macro_rules! alu_op_base {
    ($struct_name:ident, $op_name:literal, $funct3:literal, $funct7:literal) => {
        #[derive(Op, OpAssembly, OpValidator)]
        #[operation(name = $op_name, dialect = riscv)]
        pub struct $struct_name {
            #[operand]
            rd: Register,
            #[operand]
            rs1: Register,
            #[operand]
            rs2: Register,
            r#impl: OpImpl,
        }

        impl BinaryEmittable for $struct_name {
            fn encode(
                &self,
                _target_opts: &tir_backend::TargetOptions,
                stream: &mut Box<dyn tir_backend::BinaryStream>,
            ) -> tir_core::Result<()> {
                let instr = RTypeInstr::builder()
                    .opcode(ALU_OPCODE)
                    .rd(assemble_reg(self.get_rd())?)
                    .funct3($funct3)
                    .rs1(assemble_reg(self.get_rs1())?)
                    .rs2(assemble_reg(self.get_rs2())?)
                    .funct7($funct7)
                    .build();
                stream.write(&instr.to_bytes());
                Ok(())
            }
        }

        impl ISAParser for $struct_name {
            fn parse(input: &mut TokenStream<'_, '_>) -> AsmPResult<()> {
                let opcode = one_of(|t| if let AsmToken::Ident(name) = t {
                    name == lowercase!($op_name) || name == uppercase!($op_name)
                } else {
                    false
                });
                let reg = one_of(|t| matches!(t, AsmToken::Ident(_))).map(|t| {
                    if let AsmToken::Ident(name) = t {
                        name
                    } else {
                        unreachable!();
                    }
                }).and_then(register_parser);
                let comma = one_of(|t| t == AsmToken::Comma).void();

                let regs: Vec<Register> = preceded(opcode, separated(3, reg, comma)).parse_next(input)?;
                let (rd, rs1, rs2) = (regs[0], regs[1], regs[2]);

                let builder = input.get_builder();
                let context = builder.get_context();
                let op = $struct_name::builder(&context).rs1(rs1).rs2(rs2).rd(rd).build();
                builder.insert(&op);

                Ok(())
            }
        }
    };
}

macro_rules! alu_imm_op_base {
    ($struct_name:ident, $op_name:literal, $funct3:literal) => {
        #[derive(Op, OpAssembly, OpValidator)]
        #[operation(name = $op_name, dialect = riscv, known_attrs(imm: IntegerAttr))]
        pub struct $struct_name {
            #[operand]
            rd: Register,
            #[operand]
            rs1: Register,
            r#impl: OpImpl,
        }

        impl BinaryEmittable for $struct_name {
            fn encode(
                &self,
                _target_opts: &tir_backend::TargetOptions,
                stream: &mut Box<dyn tir_backend::BinaryStream>,
            ) -> tir_core::Result<()> {
                let instr = ITypeInstr::builder()
                    .opcode(ALU_IMM_OPCODE)
                    .rd(assemble_reg(self.get_rd())?)
                    .funct3($funct3)
                    .rs1(assemble_reg(self.get_rs1())?)
                    .imm(
                        self.get_imm_attr()
                            .try_into()
                            .map_err(|_| tir_core::Error::Unknown)?,
                    )
                    .build();
                stream.write(&instr.to_bytes());
                Ok(())
            }
        }

        impl ISAParser for $struct_name {
            fn parse(input: &mut TokenStream<'_, '_>) -> AsmPResult<()> {
                let opcode = one_of(|t| if let AsmToken::Ident(name) = t {
                    name == lowercase!($op_name) || name == uppercase!($op_name)
                } else {
                    false
                });
                let reg = one_of(|t| matches!(t, AsmToken::Ident(_))).map(|t| {
                    if let AsmToken::Ident(name) = t {
                        name
                    } else {
                        unreachable!();
                    }
                }).and_then(register_parser);
                let comma1 = one_of(|t| t == AsmToken::Comma).void();
                let comma2 = one_of(|t| t == AsmToken::Comma).void();
                let imm = one_of(|t| matches!(t, AsmToken::Number(_))).map(|t| match t {
                    AsmToken::Number(num) => num as i16,
                    _ => unreachable!("Why is this not a number>"),
                });

                let args: (Vec<Register>, i16) = preceded(opcode, separated_pair(separated(2, reg, comma1), comma2, imm)).parse_next(input)?;
                let (rd, rs1, imm) = (args.0[0], args.0[1], args.1);

                let builder = input.get_builder();
                let context = builder.get_context();
                let op = $struct_name::builder(&context).rs1(rs1).imm(imm.into()).rd(rd).build();
                builder.insert(&op);

                Ok(())
            }
        }
    };
}

macro_rules! alu_ops {
    // R-format ALU operations
    ($($struct_name:ident => { name = $op_name:literal, funct7 = $funct7:literal, funct3 = $funct3:literal })*) => {
        $(
        alu_op_base!($struct_name, $op_name, $funct3, $funct7);
        )*

        pub fn disassemble_alu_instr(context: &ContextRef, stream: &[u8]) -> Option<OpRef> {
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
                    let op = $struct_name::builder(&context).rs1(rs1).rs2(rs2).rd(rd).build();
                    Some(op)
                },
                )*
                _ => None,
            }
        }
    };

    ($($struct_name:ident => { name = $op_name:literal, funct3 = $funct3:literal })*) => {
        $(
        alu_imm_op_base!($struct_name, $op_name, $funct3);
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

alu_ops! {
    AddImmOp => { name = "addi", funct3 = 0b000 }
    SllImmOp => { name = "slli", funct3 = 0b001 }
    SltImmOp => { name = "slti", funct3 = 0b010 }
    SltuImmOp => { name = "sltiu", funct3 = 0b011 }
    SrlImmOp => { name = "srli", funct3 = 0b101 }
    SraImmOp => { name = "srai", funct3 = 0b101 }
    OrImmOp => { name = "ori", funct3 = 0b110 }
    AndImmOp => { name = "andi", funct3 = 0b111 }
}

isema::def! {dialect = riscv, AddOp => tir_backend::isema::AddOp{rd = get_rd, rs1 = get_rs1, rs2 = get_rs2}}
isema::def! {dialect = riscv, SubOp => tir_backend::isema::SubOp{rd = get_rd, rs1 = get_rs1, rs2 = get_rs2}}
isema::def! {dialect = riscv, AndOp => tir_backend::isema::AndOp{rd = get_rd, rs1 = get_rs1, rs2 = get_rs2}}
isema::def! {dialect = riscv, OrOp => tir_backend::isema::OrOp{rd = get_rd, rs1 = get_rs1, rs2 = get_rs2}}
isema::def! {dialect = riscv, SllOp => tir_backend::isema::SllOp{rd = get_rd, rs1 = get_rs1, rs2 = get_rs2}}
isema::def! {dialect = riscv, SrlOp => tir_backend::isema::SrlOp{rd = get_rd, rs1 = get_rs1, rs2 = get_rs2}}
isema::def! {dialect = riscv, SraOp => tir_backend::isema::SraOp{rd = get_rd, rs1 = get_rs1, rs2 = get_rs2}}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disassemble_alu_instr;
    use std::any::TypeId;

    use tir_backend::isema::convert_to_isema;
    use tir_core::{builtin::ModuleOp, Context};

    #[test]
    fn test_alu_disassembler() {
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
            0x00730e33_u32,
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
        context.add_dialect(crate::create_dialect());

        let mut ops = vec![];

        for instr in instructions {
            if let Some(op) = disassemble_alu_instr(&context, &instr.to_le_bytes()) {
                ops.push(op);
            }
        }

        assert_eq!(ops.len(), 9);
        // assert_eq!(ops[0].borrow().type_id(), TypeId::of::<AddOp>());
        assert_eq!(ops[1].borrow().type_id(), TypeId::of::<SubOp>());
        assert_eq!(ops[2].borrow().type_id(), TypeId::of::<SllOp>());
        assert_eq!(ops[3].borrow().type_id(), TypeId::of::<SltOp>());
        assert_eq!(ops[4].borrow().type_id(), TypeId::of::<SltuOp>());
        assert_eq!(ops[5].borrow().type_id(), TypeId::of::<SrlOp>());
        assert_eq!(ops[6].borrow().type_id(), TypeId::of::<SraOp>());
        assert_eq!(ops[7].borrow().type_id(), TypeId::of::<OrOp>());
        assert_eq!(ops[8].borrow().type_id(), TypeId::of::<AndOp>());
    }

    #[test]
    fn test_alu_disassembler_negative() {
        // _boot:
        //   addi x28, x6, 1000
        //   jal _boot
        // some bogus instr
        let instructions = vec![0x3e830e13_u32, 0xffdff0ef, 0x7fffff3];

        let context = Context::new();
        context.add_dialect(crate::create_dialect());

        let mut ops = vec![];

        for instr in instructions {
            if let Some(op) = disassemble_alu_instr(&context, &instr.to_le_bytes()) {
                ops.push(op);
            }
        }

        assert_eq!(ops.len(), 0);
    }

    #[test]
    fn test_sema() {
        let context = Context::new();
        context.add_dialect(crate::create_dialect());
        context.add_dialect(tir_backend::target::create_dialect());
        context.add_dialect(tir_backend::isema::create_dialect());

        let module = ModuleOp::builder(&context).build();

        let builder = OpBuilder::new(context.clone(), module.borrow().get_body());

        let add = AddOp::builder(&context)
            .rd(Register::X0.into())
            .rs1(Register::X0.into())
            .rs2(Register::X0.into())
            .build();
        builder.insert(&add);

        assert!(convert_to_isema(&module).is_ok());
    }
}
