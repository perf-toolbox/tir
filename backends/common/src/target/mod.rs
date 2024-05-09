use tir_core::Dialect;
use tir_core::OpAssembly;

mod ops;

pub use ops::*;
use tir_macros::{dialect, populate_dialect_ops, populate_dialect_types};

dialect!(target);
populate_dialect_ops!(SectionOp);
populate_dialect_types!();
