use std::cell::RefCell;
use std::rc::Rc;
use tir_backend::DisassemblerError;
use tir_core::{builtin::ModuleOp, Context, Dialect, Op, Operation};

mod ops;
mod registers;
pub mod utils;
pub use ops::*;
pub use registers::*;

use tir_macros::{dialect, populate_dialect_ops, populate_dialect_types};

dialect!(riscv);
populate_dialect_ops!(
    // R-type ALU ops
    AddOp, SubOp, SllOp, SltOp, SltuOp, SrlOp, SraOp, OrOp, AndOp,
);
populate_dialect_types!();

pub fn disassemble(
    context: &Rc<RefCell<Context>>,
    stream: &[u8],
) -> Result<ModuleOp, DisassemblerError> {
    if stream.len() % 4 != 0 {
        return Err(DisassemblerError::UnexpectedEndOfStream(4, stream.len() % 4));
    }
    
    for i in 0..(stream.len() / 4) {
        let offset = i * 4;
        if let Some(_op) = disassemble_alu_instr(context, &stream[offset..]) {
            // TODO attach operation
        } else {
            // FIXME add an appropriate error
            return Err(DisassemblerError::Unknown);
        }
    }
    Err(DisassemblerError::Unknown)
}
