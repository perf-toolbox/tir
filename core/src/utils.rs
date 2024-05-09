use std::{any::TypeId, cell::RefCell, rc::Rc};

use crate::{Op, OpRef};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TraitId(TypeId);

pub fn trait_id<T: ?Sized + 'static>() -> TraitId {
    TraitId(TypeId::of::<T>())
}

pub fn op_cast<T: Op>(op: OpRef) -> Option<Rc<RefCell<T>>> {
    if op.borrow().type_id() != TypeId::of::<T>() {
        return None;
    }

    Some(unsafe { Rc::from_raw(Rc::into_raw(op) as *const RefCell<T>) })
}

#[cfg(test)]
mod tests {
    use crate::utils::*;
    fn has_trait<T: ?Sized + 'static>() -> bool {
        let vector = [trait_id::<dyn Send>(), trait_id::<dyn Sync>()];

        vector.iter().any(|x| x == &trait_id::<T>())
    }

    #[test]
    fn basic_use() {
        assert_ne!(trait_id::<dyn Sync>(), trait_id::<dyn Send>());

        assert!(has_trait::<dyn Send>());
    }
}
