use crate::utils::{ITypeInstr, STypeInstr};
use crate::{assemble_reg, disassemble_gpr};
use crate::{register_parser, DiagKind, Register};
use tir_backend::isema::WithISema;
use tir_backend::parser::{asm_ident, close_paren, comma, number, open_paren};
use tir_backend::BinaryEmittable;
use tir_backend::ISAParser;
use tir_backend::TokenStream;
use tir_core::OpAssembly;
use tir_core::*;
use tir_macros::{lowercase, uppercase};
use tir_macros::{Op, OpAssembly, OpValidator};

use lpl::{Diagnostic, ParseResult, ParseStream, Parser};

use crate::DIALECT_NAME;

const LOAD_OPCODE: u8 = 0b0000011;
const STORE_OPCODE: u8 = 0b0100011;

macro_rules! load_op_base {
    ($struct_name:ident, $op_name:literal, $funct3:literal, $width:literal, $sign_extend:literal) => {
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
                    .imm(
                        self.get_offset_attr()
                            .try_into()
                            .map_err(|_| tir_core::Error::Unknown)?,
                    )
                    .build();
                stream.write(&instr.to_bytes());
                Ok(())
            }
        }

        #[tir_macros::op_implements(dialect = riscv)]
        impl WithISema for $struct_name {
            fn convert(&self, builder: &OpBuilder) {
                let context = self.get_context();
                let op = tir_backend::isema::LoadOp::builder(&context)
                    .dst(self.get_rd().into())
                    .base_addr(self.get_rs1().into())
                    .offset(self.get_offset_attr().clone())
                    .width($width.into())
                    .sign_extend($sign_extend.into())
                    .build();
                builder.insert(&op);
            }
        }

        impl ISAParser for $struct_name {
            fn parse(input: TokenStream) -> ParseResult<TokenStream, ()> {
                let asm_ctx = input.get_extra().unwrap().clone();

                let opcode = asm_ident().try_map(|t, s| match t {
                    lowercase!($op_name) | uppercase!($op_name) => Ok(()),
                    _ => Err(Into::<Diagnostic>::into(DiagKind::UnknownOpcode(s))),
                });

                let reg = move || {
                    asm_ident()
                        .try_map(|r, s| {
                            register_parser(r).ok_or(Into::<Diagnostic>::into(
                                DiagKind::UnknownRegister(r.to_string(), s),
                            ))
                        })
                        .label("register")
                };

                let offset = number().map(|num| num as i16).label("offset");

                let addr = offset
                    .and_then(open_paren())
                    .and_then(reg())
                    .and_then(close_paren())
                    .map(|(((offset, _), reg), _)| (offset, reg))
                    .label("address");

                let parser = opcode
                    .and_then(reg())
                    .and_then(comma())
                    .and_then(addr)
                    .map(|(((_, rd), _), (offset, ra))| (rd, offset, ra))
                    .label($op_name);

                let ((rd, offset_value, base_reg), ni) = parser.parse(input)?;

                let builder = asm_ctx.get_builder();
                let context = builder.get_context();
                let op = $struct_name::builder(&context)
                    .rs1(base_reg)
                    .rd(rd)
                    .offset(offset_value.into())
                    .build();
                builder.insert(&op);

                Ok(((), ni))
            }
        }
    };
}

macro_rules! load_ops {
    // I-format Load operations
    ($($struct_name:ident => { name = $op_name:literal, funct3 = $funct3:literal, width = $width:literal, sign_extend = $sign_extend:literal })*) => {
        $(
        load_op_base!($struct_name, $op_name, $funct3, $width, $sign_extend);
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

macro_rules! store_op_base {
    ($struct_name:ident, $op_name:literal, $funct3:literal, $width:literal) => {
        #[derive(Op, OpAssembly, OpValidator)]
        #[operation(name = $op_name, dialect = riscv, known_attrs(offset: IntegerAttr))]
        pub struct $struct_name {
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
                let instr = STypeInstr::builder()
                    .opcode(STORE_OPCODE)
                    .funct3($funct3)
                    .rs1(assemble_reg(self.get_rs1())?)
                    .rs2(assemble_reg(self.get_rs2())?)
                    .imm(
                        self.get_offset_attr()
                            .try_into()
                            .map_err(|_| tir_core::Error::Unknown)?,
                    )
                    .build();
                stream.write(&instr.to_bytes());
                Ok(())
            }
        }

        #[tir_macros::op_implements(dialect = riscv)]
        impl WithISema for $struct_name {
            fn convert(&self, builder: &OpBuilder) {
                let context = self.get_context();
                let op = tir_backend::isema::StoreOp::builder(&context)
                    .src(self.get_rs2().into())
                    .base_addr(self.get_rs1().into())
                    .offset(self.get_offset_attr().clone())
                    .width($width.into())
                    .build();
                builder.insert(&op);
            }
        }

        impl ISAParser for $struct_name {
            fn parse(input: TokenStream) -> ParseResult<TokenStream, ()> {
                let asm_ctx = input.get_extra().unwrap().clone();

                let opcode = asm_ident().try_map(|t, s| match t {
                    lowercase!($op_name) | uppercase!($op_name) => Ok(()),
                    _ => Err(Into::<Diagnostic>::into(DiagKind::UnknownOpcode(s))),
                });

                let reg = move || {
                    asm_ident()
                        .try_map(|r, s| {
                            register_parser(r).ok_or(Into::<Diagnostic>::into(
                                DiagKind::UnknownRegister(r.to_string(), s),
                            ))
                        })
                        .label("register")
                };

                let offset = number().map(|num| num as i16).label("offset");

                let addr = offset
                    .and_then(open_paren())
                    .and_then(reg())
                    .and_then(close_paren())
                    .map(|(((offset, _), reg), _)| (offset, reg))
                    .label("address");

                let parser = opcode
                    .and_then(reg())
                    .and_then(comma())
                    .and_then(addr)
                    .map(|(((_, rd), _), (offset, ra))| (rd, offset, ra))
                    .label($op_name);

                let ((rs2, offset_value, base_reg), ni) = parser.parse(input)?;

                let builder = asm_ctx.get_builder();
                let context = builder.get_context();
                let op = $struct_name::builder(&context)
                    .rs1(base_reg)
                    .rs2(rs2)
                    .offset(offset_value.into())
                    .build();
                builder.insert(&op);

                Ok(((), ni))
            }
        }
    };
}

macro_rules! store_ops {
    // I-format Load operations
    ($($struct_name:ident => { name = $op_name:literal, funct3 = $funct3:literal, width = $width:literal })*) => {
        $(
        store_op_base!($struct_name, $op_name, $funct3, $width);
        )*

        pub fn disassemble_store_instr(context: &ContextRef, stream: &[u8]) -> Option<OpRef> {
            if stream.len() < 4 {
                return None;
            }

            let instr = STypeInstr::from_bytes(&stream[0..4].try_into().unwrap());
            if instr.opcode() != STORE_OPCODE {
                return None;
            }

            let rs1 = disassemble_gpr(instr.rs1())?;
            let rs2 = disassemble_gpr(instr.rs2())?;
            let imm = instr.imm();

            match instr.funct3() {
                $(
                $funct3 => {
                    let op = $struct_name::builder(&context).rs1(rs1).rs2(rs2).offset(imm.into()).build();
                    Some(op)
                },
                )*
                _ => None,
            }
        }
    }
}

load_ops! {
    LoadByte => {name = "lb", funct3 = 0b000, width = 8, sign_extend = true }
    LoadHalfword => {name = "lh", funct3 = 0b001, width = 16, sign_extend = true }
    LoadWord => {name = "lw", funct3 = 0b010, width = 32, sign_extend = true }
    LoadDouble => {name = "ld", funct3 = 0b011, width = 64, sign_extend = true }
    LoadByteUnsigned => {name = "lbu", funct3 = 0b100, width = 8, sign_extend = false }
    LoadHalfwordUnsigned => {name = "lhu", funct3 = 0b101, width = 16, sign_extend = false }
    LoadWordUnsigned => {name = "lwu", funct3 = 0b110, width = 32, sign_extend = false }
}

store_ops! {
    StoreByte => {name = "sb", funct3 = 0b000, width = 8 }
    StoreHalfword => {name = "sh", funct3 = 0b001, width = 16 }
    StoreWord => {name = "sw", funct3 = 0b010, width = 32 }
    StoreDouble => {name = "sd", funct3 = 0b011, width = 64 }
}
