use std::cell::RefCell;
use std::rc::Rc;
use tir_core::utils::{trait_id, TraitId};
use tir_core::*;
use tir_macros::operation;

use crate::target::DIALECT_NAME;

#[operation(name = "section")]
pub struct SectionOp {
    #[cfg(attribute = true)]
    name: String,
    #[cfg(region = true)]
    body: Type,
}

#[operation(name = "const_data")]
pub struct ConstDataOp {
    #[cfg(attribute = true)]
    name: String,
    #[cfg(attribute = true)]
    data: Vec<u8>,
}

#[operation(name = "func")]
pub struct FuncOp {
    #[cfg(attribute = true)]
    name: String,
}
