use std::cell::RefCell;
use std::rc::Rc;
use tir_backend::DisassemblerError;
use tir_core::{Context, Dialect, Op, OpBuilderRef, Operation};

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
    builder: OpBuilderRef,
    stream: &[u8],
) -> Result<(), DisassemblerError> {
    if stream.len() % 4 != 0 {
        return Err(DisassemblerError::UnexpectedEndOfStream(
            4,
            stream.len() % 4,
        ));
    }

    for i in 0..(stream.len() / 4) {
        let offset = i * 4;
        if let Some(op) = disassemble_alu_instr(context, &stream[offset..]) {
            builder.borrow_mut().insert(op);
        } else {
            // FIXME add an appropriate error
            return Err(DisassemblerError::Unknown);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use tir_core::builtin::ModuleOp;
    use tir_core::{Context, OpBuilder};
    use super::*;

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

        let mut data = vec![];

        for i in instructions {
            data.extend_from_slice(&i.to_le_bytes());
        }

        let context = Context::new();
        context.borrow_mut().add_dialect(crate::create_dialect());

        let module = ModuleOp::new(context.clone());

        let builder = OpBuilder::new(context.clone(), module.get_body());

        assert!(disassemble(&context, builder, &data).is_ok());

        let ops = module.get_body().borrow().operations.to_vec();

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
        let instructions = vec![
            0x7fffff3 as u32,
        ];

        let mut data = vec![];

        for i in instructions {
            data.extend_from_slice(&i.to_le_bytes());
        }

        let context = Context::new();
        context.borrow_mut().add_dialect(crate::create_dialect());

        let module = ModuleOp::new(context.clone());

        let builder = OpBuilder::new(context.clone(), module.get_body());

        assert!(disassemble(&context, builder, &data).is_err());
    }
}
