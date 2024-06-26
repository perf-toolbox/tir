use crate::Dialect;
use crate::Ty;

mod arith;
mod func;
mod module;
mod types;

pub use arith::*;
pub use func::*;
pub use module::*;
use tir_macros::dialect;
use tir_macros::populate_dialect_ops;
use tir_macros::populate_dialect_types;
pub use types::*;

use crate::assembly::OpAssembly;
use crate::assembly::TyAssembly;

dialect!(builtin);
populate_dialect_ops!(ModuleOp, ModuleEndOp, FuncOp, ReturnOp, ConstOp);
populate_dialect_types!(FuncType, VoidType, IntType);
