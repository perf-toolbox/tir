use std::cell::RefCell;
use std::rc::Rc;

use crate::target::DIALECT_NAME;
use tir_core::{Attr, Context, Op, Operation, OperationImpl, Region, Type};
use tir_macros::operation;

#[operation(name = "section")]
pub struct SectionOp {
    #[cfg(region = true)]
    body: Type,
}
