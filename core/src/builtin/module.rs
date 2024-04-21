use std::{cell::RefCell, rc::Rc};

use crate::builtin::DIALECT_NAME;
use crate::utils::{trait_id, TraitId};
use crate::*;
use tir_macros::operation;

#[operation(name = "module")]
pub struct ModuleOp {
    #[cfg(region = true, single_block = true)]
    body: Region,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_module() {
        assert!(ModuleOp::get_operation_name() == "module");

        let context = Context::new();
        let module = ModuleOp::builder(context).build();
        module.borrow().get_body_region();
        module.borrow().get_body();
        module.borrow().get_region();
    }
}
