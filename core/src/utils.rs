use std::{any::TypeId, cell::RefCell, rc::Rc};

use crate::{Op, OpRef};

#[doc(hidden)]
pub struct CastableMeta {
    pub type_id: TypeId,
    pub caster: *const (),
}

unsafe impl Sync for CastableMeta {}
unsafe impl Send for CastableMeta {}

pub fn op_cast<T: Op>(op: OpRef) -> Option<Rc<RefCell<T>>> {
    if op.borrow().type_id() != TypeId::of::<T>() {
        return None;
    }

    Some(unsafe { Rc::from_raw(Rc::into_raw(op) as *const RefCell<T>) })
}

/// Cast an operation to a registered trait
///
/// Rust does not allow trait upcasting (at least for now). To be able to cast
/// from `dyn Op` to user-defined trait, the trait must be globally registered.
/// That is easily done with `tir_macros::op_implements` attribute:
/// ```
/// # use tir_macros::{Op, OpAssembly, OpValidator};
/// # use winnow::Parser;
/// # use tir_core::{Op, OpAssembly, OpRef, OpImpl, Printable};
/// # use tir_core::builtin::DIALECT_NAME;
/// # #[derive(Op, Debug, Clone, OpAssembly, OpValidator)]
/// # #[operation(name = "test", dialect = test)]
/// # pub struct TestOp {
/// #   r#impl: OpImpl,
/// # }
/// #
/// use tir_core::Terminator;
///
/// #[tir_macros::op_implements(dialect = test)]
/// impl Terminator for TestOp {}
/// ```
///
/// And later users can dynamically query known traits and perform generic
/// transformations or analysis on operations.
///
/// **Example:**
/// ```
/// use tir_core::{OpRef, utils::op_dyn_cast, Terminator};
///
/// fn test(op: OpRef) {
///     if let Some(op) = op_dyn_cast::<dyn Terminator>(op) {
///         // do smth useful
///     }
/// }
/// ```
pub fn op_dyn_cast<T: ?Sized + 'static>(op: OpRef) -> Option<Rc<RefCell<T>>> {
    let meta = op.borrow().get_meta();
    let entry = meta.iter().find_map(|func| {
        let entry = func();
        if entry.type_id == TypeId::of::<RefCell<T>>() {
            Some(entry)
        } else {
            None
        }
    })?;
    let context = op.borrow().get_context();
    let op_ref = context.get_op(op.borrow().get_alloc_id())?;
    let caster =
        unsafe { std::mem::transmute::<*const (), fn(OpRef) -> Rc<RefCell<T>>>(entry.caster) };
    Some(caster(op_ref))
}

/// Check if an operation was registered to implement trait T
///
/// See [`op_dyn_cast`] for more info on registering traits.
///
/// **Example:**
/// ```
/// use tir_core::{OpRef, utils::op_has_trait, Terminator};
///
/// fn test(op: OpRef) {
///     if op_has_trait::<dyn Terminator>(op) {
///         // do smth useful
///     }
/// }
/// ```
pub fn op_has_trait<T: ?Sized + 'static>(op: OpRef) -> bool {
    op.borrow().has_trait(TypeId::of::<RefCell<T>>())
}
