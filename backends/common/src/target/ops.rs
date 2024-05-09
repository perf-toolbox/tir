use tir_core::*;
use tir_macros::{Op, OpAssembly};
use winnow::Parser;

use crate::target::DIALECT_NAME;

#[derive(Op, OpAssembly)]
#[operation(name = "section", known_attrs(name: String))]
pub struct SectionOp {
    #[region]
    body: RegionRef,
    r#impl: OpImpl,
}
