use crate::Dialect;
use crate::Ty;

mod arith;
mod func;
mod module;
mod types;
mod value;

pub use arith::*;
pub use func::*;
pub use module::*;
pub use types::*;
pub use value::*;
use tir_macros::dialect;
use tir_macros::populate_dialect_ops;
use tir_macros::populate_dialect_types;

use crate::assembly::Assembly;

dialect!(builtin);
populate_dialect_ops!(ModuleOp, FuncOp, ConstOp);
populate_dialect_types!(FuncType, VoidType);
