use crate::Dialect;
use crate::Op;
use crate::Ty;

mod func;
mod module;
mod types;

pub(crate) const DIALECT_NAME: &str = "builtin";

pub use func::*;
pub use module::*;
use tir_macros::populate_dialect_ops;
use tir_macros::populate_dialect_types;
pub use types::*;

pub fn create_dialect() -> Dialect {
    let mut dialect = Dialect::new(DIALECT_NAME);

    populate_dialect_ops!(dialect, ModuleOp, FuncOp);
    populate_dialect_types!(dialect, FuncType, VoidType);

    dialect
}
