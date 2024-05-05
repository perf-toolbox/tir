// use tir_backend::DisassemblerError;
// use tir_core::{ContextRef, IRAssembly};
use tir_core::Dialect;

mod ops;
mod registers;
pub mod utils;
pub use ops::*;
pub use registers::*;

use tir_macros::{dialect, populate_dialect_ops, populate_dialect_types};

dialect!(riscv);
populate_dialect_ops!(
    // R-type ALU ops
    // AddOp, SubOp, SllOp, SltOp, SltuOp, SrlOp, SraOp, OrOp, AndOp,
);
populate_dialect_types!();
//
// pub fn disassemble(
//     context: &ContextRef,
//     builder: OpBuilderRef,
//     stream: &[u8],
// ) -> Result<(), DisassemblerError> {
//     if stream.len() % 4 != 0 {
//         return Err(DisassemblerError::UnexpectedEndOfStream(
//             4,
//             stream.len() % 4,
//         ));
//     }
//
//     for i in 0..(stream.len() / 4) {
//         let offset = i * 4;
//         if let Some(op) = disassemble_alu_instr(context, &stream[offset..]) {
//             builder.borrow_mut().insert(op);
//         } else {
//             // FIXME add an appropriate error
//             return Err(DisassemblerError::Unknown);
//         }
//     }
//
//     Ok(())
// }
//
// #[cfg(test)]
// mod tests {
//     use std::any::TypeId;
//
//     use super::*;
//     use tir_core::builtin::ModuleOp;
//     use tir_core::{Context, OpBuilder};
//
//     #[test]
//     fn test_disassembler() {
//         // add x28, x6, x7
//         // sub x28, x6, x7
//         // sll x28, x6, x7
//         // slt x28, x6, x7
//         // sltu x28, x6, x7
//         // srl x28, x6, x7
//         // sra x28, x6, x7
//         // or x28, x6, x7
//         // and x28, x6, x7
//         let instructions = vec![
//             0x00730e33_u32,
//             0x40730e33,
//             0x00731e33,
//             0x00732e33,
//             0x00733e33,
//             0x00735e33,
//             0x40735e33,
//             0x00736e33,
//             0x00737e33,
//         ];
//
//         let mut data = vec![];
//
//         for i in instructions {
//             data.extend_from_slice(&i.to_le_bytes());
//         }
//
//         let context = Context::new();
//         context.add_dialect(crate::create_dialect());
//
//         let module = ModuleOp::builder(context.clone()).build();
//
//         let builder = OpBuilder::new(context.clone(), module.borrow_mut().get_body());
//
//         assert!(disassemble(&context, builder, &data).is_ok());
//
//         let ops = module.borrow_mut().get_body().borrow().operations.to_vec();
//
//         assert_eq!(ops.len(), 9);
//         assert_eq!(ops[0].borrow().type_id(), TypeId::of::<AddOp>());
//         assert_eq!(ops[1].borrow().type_id(), TypeId::of::<SubOp>());
//         assert_eq!(ops[2].borrow().type_id(), TypeId::of::<SllOp>());
//         assert_eq!(ops[3].borrow().type_id(), TypeId::of::<SltOp>());
//         assert_eq!(ops[4].borrow().type_id(), TypeId::of::<SltuOp>());
//         assert_eq!(ops[5].borrow().type_id(), TypeId::of::<SrlOp>());
//         assert_eq!(ops[6].borrow().type_id(), TypeId::of::<SraOp>());
//         assert_eq!(ops[7].borrow().type_id(), TypeId::of::<OrOp>());
//         assert_eq!(ops[8].borrow().type_id(), TypeId::of::<AndOp>());
//     }
//
//     #[test]
//     fn test_disassembler_negative() {
//         let instructions = vec![0x7fffff3_u32];
//
//         let mut data = vec![];
//
//         for i in instructions {
//             data.extend_from_slice(&i.to_le_bytes());
//         }
//
//         let context = Context::new();
//         context.add_dialect(crate::create_dialect());
//
//         let module = ModuleOp::builder(context.clone()).build();
//
//         let builder = OpBuilder::new(context.clone(), module.borrow().get_body());
//
//         assert!(disassemble(&context, builder, &data).is_err());
//     }
// }
