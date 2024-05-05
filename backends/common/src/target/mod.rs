use tir_core::Assembly;
use tir_core::Dialect;

mod ops;

pub use ops::*;
use tir_macros::{dialect, populate_dialect_ops, populate_dialect_types};

dialect!(target);
populate_dialect_ops!(SectionOp);
populate_dialect_types!();
