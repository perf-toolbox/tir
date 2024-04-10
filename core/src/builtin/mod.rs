use crate::Dialect;
use crate::Op;
use crate::Ty;

mod func;
mod module;
mod types;

pub(crate) const DIALECT_NAME: &str = "builtin";

pub use func::*;
pub use module::*;
pub use types::*;

pub fn create_dialect() -> Dialect {
    let mut dialect = Dialect::new(DIALECT_NAME);

    dialect.add_operation(ModuleOp::get_operation_name());
    dialect.add_operation(FuncOp::get_operation_name());

    dialect.add_type(FuncType::get_type_name());
    dialect.add_type(VoidType::get_type_name());

    dialect
}
