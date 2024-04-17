use tir_core::{Dialect, Op, Operation};

mod ops;

pub use ops::*;
use tir_macros::{dialect, populate_dialect_ops, populate_dialect_types};

dialect!(target);
populate_dialect_ops!(SectionOp, ConstDataOp, FuncOp);
populate_dialect_types!();
