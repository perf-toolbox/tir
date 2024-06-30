use std::{cell::RefCell, rc::Rc};

use tir_core::{builtin::ModuleOp, dfs_walk, utils::op_dyn_cast, Op, OpBuilder};

pub trait WithISema: tir_core::Op {
    fn convert(&self, builder: &tir_core::OpBuilder);
}

pub fn convert_to_isema(op: Rc<RefCell<ModuleOp>>) {
    let builder = OpBuilder::new(op.borrow().get_context(), op.borrow().get_body());
    dfs_walk(op, |cand| {
        if let Some(isema) = op_dyn_cast::<dyn WithISema>(cand.clone()) {
            builder.set_insertion_point_after(&isema);
            isema.borrow().convert(&builder);
        }
    });
}
