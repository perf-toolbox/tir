use tir_backend::DisassemblerError;
use tir_core::{builtin::ModuleOp, Dialect, Op, Operation};

mod ops;
mod registers;
pub use ops::*;
pub use registers::*;

use tir_macros::{dialect, populate_dialect_ops, populate_dialect_types};

dialect!(riscv);
populate_dialect_ops!(
    // R-type ALU ops
    AddOp, SubOp, SllOp, SltOp, SltuOp, SrlOp, SraOp, OrOp, AndOp,
);
populate_dialect_types!();

pub fn disassemble(_stream: &[u8]) -> Result<ModuleOp, DisassemblerError> {
    Err(DisassemblerError::Unknown)
}
