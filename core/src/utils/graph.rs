use std::any::Any;
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

pub struct Borrow<'a, T: ?Sized> {
    ptr: *const T,
    #[allow(dead_code)]
    lock: RwLockReadGuard<'a, ()>,
}

pub struct BorrowMut<'a, T: ?Sized> {
    ptr: *mut T,
    #[allow(dead_code)]
    lock: RwLockWriteGuard<'a, ()>,
}

impl<'a, T> Deref for Borrow<'a, T>
where
    T: ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // TODO add refcount and make sure pointer is still alive
        unsafe { self.ptr.as_ref() }.unwrap()
    }
}

impl<'a, T> Deref for BorrowMut<'a, T>
where
    T: ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // TODO add refcount and make sure pointer is still alive
        unsafe { self.ptr.as_ref() }.unwrap()
    }
}

impl<'a, T> DerefMut for BorrowMut<'a, T>
where
    T: ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        // TODO add refcount and make sure pointer is still alive
        unsafe { self.ptr.as_mut() }.unwrap()
    }
}

pub struct NodeRef<T: ?Sized + Any> {
    ptr: *mut T,
    lock: Rc<RwLock<()>>,
}

impl<T> Clone for NodeRef<T>
where
    T: ?Sized + Any,
{
    fn clone(&self) -> Self {
        NodeRef {
            ptr: self.ptr.clone(),
            lock: self.lock.clone(),
        }
    }
}

impl<T: ?Sized + Any> NodeRef<T> {
    pub fn borrow(&self) -> Borrow<'_, T> {
        match self.lock.read() {
            Ok(lock) => Borrow {
                ptr: self.ptr,
                lock,
            },
            Err(_) => {
                panic!("Failed to acquire read lock")
            }
        }
    }

    pub fn borrow_mut(&self) -> BorrowMut<'_, T> {
        match self.lock.write() {
            Ok(lock) => BorrowMut {
                ptr: self.ptr,
                lock,
            },
            Err(_) => {
                panic!("Failed to acquire read lock")
            }
        }
    }
}

pub struct NodeBox<T: ?Sized + Any> {
    ptr: *mut T,
    lock: Rc<RwLock<()>>,
    parent: Option<GraphPtr<dyn GraphRoot<T>, T>>,
}

impl<T> Clone for NodeBox<T>
where
    T: ?Sized + Any,
{
    fn clone(&self) -> Self {
        NodeBox {
            ptr: self.ptr.clone(),
            lock: self.lock.clone(),
            parent: self.parent.clone(),
        }
    }
}

impl<T> NodeBox<T>
where
    T: ?Sized + Any,
{
    pub fn new(data: Box<T>, parent: Option<GraphPtr<dyn GraphRoot<T>, T>>) -> Self {
        NodeBox {
            ptr: Box::into_raw(data),
            lock: Rc::new(RwLock::new(())),
            parent,
        }
    }

    pub fn as_ref(&self) -> NodeRef<T> {
        NodeRef {
            ptr: self.ptr,
            lock: self.lock.clone(),
        }
    }

    pub fn get_parent(&self) -> Option<GraphPtr<dyn GraphRoot<T>, T>> {
        self.parent.clone()
    }
}

// FIXME: we shouldn't need Drop for NodeBox once we have arenas fully implemented.
impl<T> Drop for NodeBox<T>
where
    T: ?Sized + Any,
{
    fn drop(&mut self) {
        unsafe {
            // Explicitly drop the memory
            let _ = Box::from_raw(self.ptr);
        }
    }
}

pub struct GraphPtr<T, Node>
where
    T: GraphRoot<Node> + ?Sized,
    Node: ?Sized + Any,
{
    ptr: *mut T,
    lock: Rc<RwLock<()>>,
    _forked_from: Option<*mut T>,
    _b: std::marker::PhantomData<Node>,
}

impl<T, Node> Clone for GraphPtr<T, Node>
where
    T: GraphRoot<Node> + ?Sized,
    Node: ?Sized + Any,
{
    fn clone(&self) -> Self {
        GraphPtr {
            ptr: self.ptr.clone(),
            lock: self.lock.clone(),
            _forked_from: self._forked_from.clone(),
            _b: PhantomData::default(),
        }
    }
}

impl<T, Node> GraphPtr<T, Node>
where
    T: GraphRoot<Node> + ?Sized,
    Node: ?Sized + Any,
{
    pub fn borrow(&self) -> Borrow<'_, T> {
        match self.lock.read() {
            Ok(lock) => Borrow {
                ptr: self.ptr,
                lock,
            },
            Err(_) => {
                panic!("Failed to acquire read lock")
            }
        }
    }

    pub fn borrow_mut(&self) -> BorrowMut<'_, T> {
        match self.lock.write() {
            Ok(lock) => BorrowMut {
                ptr: self.ptr,
                lock,
            },
            Err(_) => {
                panic!("Failed to acquire read lock")
            }
        }
    }
}

pub struct GraphBox<T, Node>
where
    T: GraphRoot<Node> + ?Sized + Any + Clone,
    Node: ?Sized + Any,
{
    ptr: *mut T,
    lock: Rc<RwLock<()>>,
    _phantom: PhantomData<Node>,
}

impl<T, Node> GraphBox<T, Node>
where
    T: GraphRoot<Node> + ?Sized + Any + Clone,
    Node: ?Sized + Any,
{
    pub fn new(data: Box<T>) -> Self {
        GraphBox {
            ptr: Box::into_raw(data),
            lock: Rc::new(RwLock::new(())),
            _phantom: PhantomData::default(),
        }
    }

    pub fn as_ref(&self) -> GraphPtr<T, Node> {
        GraphPtr {
            ptr: self.ptr.clone(),
            lock: self.lock.clone(),
            _forked_from: None,
            _b: PhantomData::default(),
        }
    }
}

pub trait GraphRoot<Node>
where
    Node: ?Sized + Any,
    Self: Any,
{
    fn allocate_node(this: &GraphPtr<Self, Node>, node_data: Box<Node>) -> NodeRef<Node>
    where
        Self: Sized;
    fn fork(this: GraphPtr<Self, Node>) -> GraphBox<Self, Node>
    where
        Self: Sized + Clone,
    {
        unimplemented!()
    }
    fn get_parent(this: GraphPtr<Self, Node>) -> Option<GraphPtr<dyn GraphRoot<Node>, Node>>
    where
        Self: Sized,
    {
        None
    }
    // fn allocate_node(this: NodeRef<Self>, node_data: Box<T>) -> NodeRef<T> where Self: Sized;
}

#[cfg(test)]
mod tests {
    use super::{GraphBox, GraphPtr, GraphRoot, NodeBox, NodeRef};
    use std::any::Any;

    trait TestOp: Any {}

    #[derive(Clone)]
    struct TestModuleOp {
        owning_ops: Vec<NodeBox<dyn TestOp>>,
    }
    struct TestChildOp {}

    impl TestOp for TestModuleOp {}
    impl TestOp for TestChildOp {}

    impl TestModuleOp {
        pub fn new() -> GraphBox<Self, dyn TestOp> {
            GraphBox::new(Box::new(TestModuleOp { owning_ops: vec![] }))
        }
    }

    impl GraphRoot<dyn TestOp> for TestModuleOp {
        fn allocate_node(
            this: &GraphPtr<Self, dyn TestOp>,
            node_data: Box<dyn TestOp>,
        ) -> NodeRef<dyn TestOp> {
            this.borrow_mut()
                .owning_ops
                .push(NodeBox::new(node_data, Some(this.clone().into())));
            unimplemented!()
        }
    }

    #[test]
    fn test() {
        let module = TestModuleOp::new();
        let _op = GraphRoot::allocate_node(&module.as_ref(), Box::new(TestChildOp {}));
    }

    // struct TestNode {
    //     pub value: u32,
    // }
    //
    // struct TestGraph {
    //     nodes: Vec<NodeBox<dyn Any>>,
    // }
    //
    // impl TestGraph {
    //     pub fn new() -> NodeBox<Self> {
    //         NodeBox::new(Box::new(TestGraph { nodes: vec![] }), None)
    //     }
    // }
    //
    // impl GraphRoot<dyn Any> for TestGraph {
    //     fn allocate_node(this: super::NodeRef<Self>, node_data: Box<dyn Any>) -> super::NodeRef<dyn Any> {
    //         this.borrow_mut().nodes.push(NodeBox::new(node_data, None));
    //         this.borrow().nodes.last().unwrap().as_ref()
    //     }
    // }
    //
    // #[test]
    // fn test_basic_borrow() {
    //     let graph_box = TestGraph::new();
    //     let graph = graph_box.as_ref();
    //     let node = GraphRoot::allocate_node(graph, Box::new(TestNode{value: 42}));
    //     assert_eq!(node.borrow().downcast_ref::<TestNode>().unwrap().value, 42);
    //     node.borrow_mut().downcast_mut::<TestNode>().unwrap().value = 0;
    //     assert_eq!(node.borrow().downcast_ref::<TestNode>().unwrap().value, 0);
    // }
}
