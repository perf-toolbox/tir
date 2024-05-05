use crate::Dialect;
use crate::Ty;
// use crate::Op;
//
mod arith;
// mod func;
mod module;
mod types;
//
// use crate::assembly::IRAssembly;
pub use arith::*;
// pub use func::*;
pub use module::*;
use tir_macros::dialect;
use tir_macros::populate_dialect_ops;
use tir_macros::populate_dialect_types;
pub use types::*;
//
dialect!(builtin);
// populate_dialect_ops!(ModuleOp, FuncOp, ConstOp);
populate_dialect_ops!(ModuleOp, ConstOp);
populate_dialect_types!(FuncType, VoidType);
