use std::cell::RefCell;
use std::rc::Rc;

use crate::builtin::FuncType;
use crate::builtin::DIALECT_NAME;
use crate::utils::{trait_id, TraitId};
use crate::*;
use tir_macros::operation;

#[operation(name = "func")]
pub struct FuncOp {
    #[cfg(region = true)]
    body: Type,
    #[cfg(attribute = true)]
    sym_name: String,
    #[cfg(attribute = true)]
    func_type: FuncType,
}
