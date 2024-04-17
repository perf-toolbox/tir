use tir_core::{Dialect, Op, Operation};

mod ops;
pub use ops::*;

use tir_macros::{dialect, populate_dialect_ops, populate_dialect_types};

dialect!(riscv);
populate_dialect_ops!();
populate_dialect_types!();
