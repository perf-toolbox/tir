use std::{any::TypeId, cell::RefCell, rc::Rc};

use crate::{Op, OpRef};

pub struct CastableMeta {
    pub type_id: TypeId,
    pub caster: *const (),
}

unsafe impl Sync for CastableMeta {}
unsafe impl Send for CastableMeta {}

pub trait OpCastable<T: ?Sized + 'static> {
    fn downcast(self) -> Rc<RefCell<T>>;
}

pub fn op_cast<T: Op>(op: OpRef) -> Option<Rc<RefCell<T>>> {
    if op.borrow().type_id() != TypeId::of::<T>() {
        return None;
    }

    Some(unsafe { Rc::from_raw(Rc::into_raw(op) as *const RefCell<T>) })
}

pub fn op_dyn_cast<T: ?Sized + 'static>(op: OpRef) -> Option<Rc<RefCell<T>>> {
    let meta = op.borrow().get_meta();
    let entry = meta.iter().find_map(|func| {
        let entry = func();
        if entry.type_id == std::any::TypeId::of::<std::cell::RefCell<T>>() {
            Some(entry)
        } else {
            None
        }
    })?;
    let context = op.borrow().get_context();
    let op_ref = context.get_op(op.borrow().get_alloc_id())?;
    let caster = unsafe {
        std::mem::transmute::<*const (), fn(OpRef) -> std::rc::Rc<std::cell::RefCell<T>>>(
            entry.caster,
        )
    };
    Some(caster(op_ref))
}

pub fn op_has_trait<T: ?Sized + 'static>(op: OpRef) -> bool {
    op.borrow().has_trait(TypeId::of::<RefCell<T>>())
}
