use std::{
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock, RwLockReadGuard, RwLockWriteGuard,
    },
};

pub struct Intrusion {
    pub ref_count: AtomicUsize,
    pub death_mark: Arc<RwLock<bool>>,
    pub lock: RwLock<()>,
}

impl Default for Intrusion {
    fn default() -> Self {
        Intrusion {
            ref_count: AtomicUsize::new(1),
            death_mark: Arc::new(RwLock::new(false)),
            lock: RwLock::new(()),
        }
    }
}

impl Intrusion {
    pub fn retain(&mut self) {
        // TODO: check if this ordering is correct
        self.ref_count.fetch_add(1, Ordering::AcqRel);
    }

    pub fn release(&mut self) -> usize {
        self.ref_count.fetch_sub(1, Ordering::AcqRel)
    }

    pub fn lock_read(&mut self) -> RwLockReadGuard<()> {
        // TODO think of panic safety
        self.lock.read().unwrap()
    }

    pub fn lock_write(&mut self) -> RwLockWriteGuard<()> {
        // TODO think of panic safety
        self.lock.write().unwrap()
    }
}

pub trait Intrusive {
    // fn intrusive_get_ref_count(&self);
    fn intrusive_retain(&mut self);
    fn intrusive_release(&mut self) -> usize;
    fn intrusive_read_lock(&mut self) -> RwLockReadGuard<()>;
    fn intrusive_write_lock(&mut self) -> RwLockWriteGuard<()>;
}

pub struct IRef<'a, T: ?Sized> {
    ptr: *const T,
    #[allow(dead_code)]
    lock: RwLockReadGuard<'a, ()>,
}

impl<'a, T> Deref for IRef<'a, T>
where
    T: ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }.unwrap()
    }
}

pub struct IRefMut<'a, T: ?Sized> {
    ptr: *mut T,
    #[allow(dead_code)]
    lock: RwLockWriteGuard<'a, ()>,
}

impl<'a, T> Deref for IRefMut<'a, T>
where
    T: ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }.unwrap()
    }
}

impl<'a, T> DerefMut for IRefMut<'a, T>
where
    T: ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }.unwrap()
    }
}

pub struct IPtr<T>
where
    T: ?Sized + Intrusive,
{
    ptr: *mut T,
}

impl<T> IPtr<T>
where
    T: ?Sized + Intrusive,
{
    pub fn new(contents: Box<T>) -> Self {
        let ptr = Box::into_raw(contents);

        IPtr { ptr }
    }

    pub fn borrow(&self) -> IRef<'_, T> {
        let lock = unsafe { self.ptr.as_mut() }.unwrap().intrusive_read_lock();
        IRef {
            ptr: self.ptr,
            lock,
        }
    }

    pub fn borrow_mut(&self) -> IRefMut<'_, T> {
        let lock = unsafe { self.ptr.as_mut() }.unwrap().intrusive_write_lock();
        IRefMut {
            ptr: self.ptr,
            lock,
        }
    }
}

impl<T> Clone for IPtr<T>
where
    T: ?Sized + Intrusive,
{
    fn clone(&self) -> Self {
        unsafe { self.ptr.as_mut() }.unwrap().intrusive_retain();
        IPtr { ptr: self.ptr }
    }
}

#[cfg(test)]
mod tests {
    use super::{IPtr, Intrusion, Intrusive};

    #[derive(Default)]
    struct TestOp {
        value: u32,
        intrusion: Intrusion,
    }

    impl Intrusive for TestOp {
        fn intrusive_release(&mut self) -> usize {
            self.intrusion.release()
        }
        fn intrusive_retain(&mut self) {
            self.intrusion.retain();
        }
        fn intrusive_read_lock(&mut self) -> std::sync::RwLockReadGuard<()> {
            self.intrusion.lock_read()
        }
        fn intrusive_write_lock(&mut self) -> std::sync::RwLockWriteGuard<()> {
            self.intrusion.lock_write()
        }
    }

    #[test]
    fn basic() {
        let op = IPtr::new(Box::new(TestOp::default()));
        assert_eq!(op.borrow().value, 0);
        op.borrow_mut().value = 42;
        assert_eq!(op.borrow().value, 42);
    }
}
