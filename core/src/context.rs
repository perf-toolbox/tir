use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::{Arc, RwLock, Weak};

use crate::{builtin, Dialect, Op, OpRef};

pub type ContextRef = Arc<Context>;
pub type ContextWRef = Weak<Context>;

/// Denotes a properly allocated operation bound to context
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
pub struct AllocId {
    id: usize,
}

impl Default for AllocId {
    fn default() -> Self {
        AllocId { id: usize::MAX }
    }
}

struct ContextImpl {
    dialects: Vec<Arc<Dialect>>,
    allocated_operations: HashMap<AllocId, OpRef>,
}

impl ContextImpl {
    fn new() -> RwLock<ContextImpl> {
        let builtin_dialect = builtin::create_dialect();
        let mut r#impl = ContextImpl {
            dialects: vec![],
            allocated_operations: HashMap::new(),
        };
        r#impl.add_dialect(builtin_dialect);
        RwLock::new(r#impl)
    }

    fn add_dialect(&mut self, dialect: Dialect) -> Arc<Dialect> {
        let mut dialect = dialect;
        dialect.set_id((self.dialects.len()).try_into().unwrap());
        self.dialects.push(Arc::new(dialect));
        let dialect_ref = self.dialects.last().unwrap();

        dialect_ref.clone()
    }

    fn get_dialect_by_name(&self, name: &str) -> Option<Arc<Dialect>> {
        for dialect in &self.dialects {
            if dialect.get_name() == name {
                return Some(dialect.clone());
            }
        }
        None
    }

    fn get_dialect(&self, id: u32) -> Option<Arc<Dialect>> {
        if id as usize > self.dialects.len() {
            None
        } else {
            Some(self.dialects[id as usize].clone())
        }
    }

    fn allocate_op<T: Op + 'static>(&mut self, op: T) -> Rc<RefCell<T>> {
        let id = AllocId {
            id: self.allocated_operations.len(),
        };

        let mut op = op;
        op.set_alloc_id(id);

        let op = Rc::new(RefCell::new(op));

        self.allocated_operations.insert(id, op.clone());

        op
    }

    pub fn get_op(&self, id: AllocId) -> Option<OpRef> {
        self.allocated_operations.get(&id).cloned()
    }
}

/// Context holds all the resources required for building an IR
///
/// Examples:
/// ```
/// use tir_core::{Dialect, Context};
///
/// fn foo(dialect_obj: Dialect) {
///     // Create a new context
///     let context = Context::new();
///     // Register dialects
///     let dialect = context.add_dialect(dialect_obj);
/// }
/// ```
pub struct Context {
    r#impl: RwLock<ContextImpl>,
}

impl Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
    {
        f.write_str("Context")?;
        Ok(())
    }
}

impl Context {
    /// Create a new context
    ///
    /// Every newly-created context will have at least two dialects registered:
    /// builtin dialect and std dialect
    pub fn new() -> Arc<Context> {
        Arc::new(Context {
            r#impl: ContextImpl::new(),
        })
    }

    /// Register a new dialect with a context
    pub fn add_dialect(&self, dialect: Dialect) -> Arc<Dialect> {
        let mut lock = self.r#impl.write().unwrap();
        lock.add_dialect(dialect)
    }

    /// Find a registered dialect by its name
    pub fn get_dialect_by_name(&self, name: &str) -> Option<Arc<Dialect>> {
        let lock = self.r#impl.read().unwrap();
        lock.get_dialect_by_name(name)
    }

    /// Get dialect by ID
    pub fn get_dialect(&self, id: u32) -> Option<Arc<Dialect>> {
        let lock = self.r#impl.read().unwrap();
        lock.get_dialect(id)
    }

    /// Take ownership of operation data and return a shared reference
    pub fn allocate_op<T: Op + 'static>(&self, op: T) -> Rc<RefCell<T>> {
        let mut lock = self.r#impl.write().unwrap();
        lock.allocate_op(op)
    }

    /// Find allocated operation in the current context and return a shared reference
    pub fn get_op(&self, id: AllocId) -> Option<OpRef> {
        let lock = self.r#impl.read().unwrap();
        lock.get_op(id)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_context() {
        let context = Context::new();
        assert!(context.get_dialect_by_name("builtin").is_some());
    }
}
