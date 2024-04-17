use tir_core::{Dialect, Op};

mod ops;

pub use ops::*;
use tir_macros::populate_dialect_ops;

pub(crate) const DIALECT_NAME: &str = "target";

pub fn create_dialect() -> Dialect {
    let mut dialect = Dialect::new(DIALECT_NAME);

    populate_dialect_ops!(dialect, SectionOp);

    dialect
}
