use std::{any::TypeId, cell::RefCell, rc::Rc};

use crate::{Op, OpRef};

pub fn op_cast<T: Op>(op: OpRef) -> Option<Rc<RefCell<T>>> {
    if op.borrow().type_id() != TypeId::of::<T>() {
        return None;
    }

    Some(unsafe { Rc::from_raw(Rc::into_raw(op) as *const RefCell<T>) })
}
