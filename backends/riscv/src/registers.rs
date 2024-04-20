#![allow(clippy::all)]

use seq_macro::seq;
use tir_core::Operand;

seq!(N in 0..=31 {
  pub enum Reg {
      #(
          X~N,
      )*
      #(
          F~N,
      )*
  }
});

impl Into<Operand> for Reg {
    fn into(self) -> Operand {
        seq!(N in 0..=31 {
          let result = match self {
              #(
              Reg::X~N => Operand::Register(N),
              )*
              #(
              Reg::F~N => Operand::Register(31 + N),
              )*
          };
        });

        result
    }
}

pub fn disassemble_gpr(value: u8) -> Option<Operand> {
    seq!(N in 0..=31 {
        let result = match value {
        #(
            N => Some(Reg::X~N.into()),
        )*
            _ => None,
        };
    });

    result
}

pub fn encode_gpr(operand: &Operand) -> tir_core::Result<u8> {
    match operand {
        Operand::Register(reg) => {
            if *reg >= 32 {
                Err(tir_core::Error::Unknown)
            } else {
                Ok(*reg as u8)
            }
        }
        _ => Err(tir_core::Error::Unknown),
    }
}
