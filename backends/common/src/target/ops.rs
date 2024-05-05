use tir_core::*;
use tir_macros::{Assembly, Op};

use crate::target::DIALECT_NAME;

#[derive(Op, Assembly)]
#[operation(name = "section", known_attrs(name: String))]
pub struct SectionOp {
    #[region]
    body: RegionRef,
    r#impl: OpImpl,
}
