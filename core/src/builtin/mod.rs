use crate::Dialect;
use crate::Op;

use crate::Ty;

mod func;
mod module;
mod types;

use crate::assembly::IRAssembly;
pub use func::*;
pub use module::*;
use tir_macros::dialect;
use tir_macros::populate_dialect_ops;
use tir_macros::populate_dialect_types;
pub use types::*;

dialect!(builtin);
populate_dialect_ops!(ModuleOp, FuncOp);
populate_dialect_types!(FuncType, VoidType);
