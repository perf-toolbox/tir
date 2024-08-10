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
use winnow::combinator::{preceded, separated};
use winnow::token::one_of;
use winnow::Parser;

use crate::DIALECT_NAME;

const LOAD_OPCODE: u8 = 0b0000011;

macro_rules! load_op_base {
    ($struct_name:ident, $op_name:literal, $funct3:literal) => {
        #[derive(Op, OpAssembly, OpValidator)]
        #[operation(name = $op_name, dialect = riscv, known_attrs(offset: IntegerAttr))]
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
                    .opcode(LOAD_OPCODE)
                    .rd(assemble_reg(self.get_rd())?)
                    .funct3($funct3)
                    .rs1(assemble_reg(self.get_rs1())?)
                    .imm(0)
                    .build();
                stream.write(&instr.to_bytes());
                Ok(())
            }
        }

        impl ISAParser for $struct_name {
            fn parse(input: &mut TokenStream<'_, '_>) -> AsmPResult<()> {
                let opcode = one_of(|t| {
                    if let AsmToken::Ident(name) = t {
                        name == lowercase!($op_name) || name == uppercase!($op_name)
                    } else {
                        false
                    }
                });
                let reg = one_of(|t| matches!(t, AsmToken::Ident(_)))
                    .map(|t| {
                        if let AsmToken::Ident(name) = t {
                            name
                        } else {
                            unreachable!();
                        }
                    })
                    .and_then(register_parser);
                let comma = one_of(|t| t == AsmToken::Comma).void();

                todo!()

                // let regs: Vec<Register> = preceded(opcode, separated(3, reg, comma)).parse_next(input)?;
                // let (rd, rs1, rs2) = (regs[0], regs[1], regs[2]);
                //
                // let builder = input.get_builder();
                // let context = builder.get_context();
                // let op = $struct_name::builder(&context).rs1(rs1).rs2(rs2).rd(rd).build();
                // builder.insert(&op);
                //
                // Ok(())
            }
        }
    };
}

macro_rules! load_ops {
    // I-format Load operations
    ($($struct_name:ident => { name = $op_name:literal, funct3 = $funct3:literal })*) => {
        $(
        load_op_base!($struct_name, $op_name, $funct3);
        )*

        pub fn disassemble_load_instr(context: &ContextRef, stream: &[u8]) -> Option<OpRef> {
            if stream.len() < 4 {
                return None;
            }

            let instr = ITypeInstr::from_bytes(&stream[0..4].try_into().unwrap());
            if instr.opcode() != LOAD_OPCODE {
                return None;
            }

            let rd = disassemble_gpr(instr.rd())?;
            let rs1 = disassemble_gpr(instr.rs1())?;
            let imm = instr.imm();

            match instr.funct3() {
                $(
                $funct3 => {
                    let op = $struct_name::builder(&context).rd(rd).rs1(rs1).offset(imm.into()).build();
                    Some(op)
                },
                )*
                _ => None,
            }
        }
    }
}

load_ops! {
    LoadByte => {name = "lb", funct3 = 0b000 }
    LoadHalfword => {name = "lh", funct3 = 0b001 }
    LoadWord => {name = "lw", funct3 = 0b010 }
    LoadDouble => {name = "ld", funct3 = 0b011 }
    LoadByteUnsigned => {name = "lbu", funct3 = 0b100 }
    LoadHalfwordUnsigned => {name = "lhu", funct3 = 0b101 }
    LoadWordUnsigned => {name = "lwu", funct3 = 0b110 }
}
