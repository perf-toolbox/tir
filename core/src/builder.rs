// use crate::{BlockRef, ContextRef, Operation};
// use std::cell::RefCell;
// use std::rc::Rc;
//
// pub type OpBuilderRef = Rc<RefCell<OpBuilder>>;
//
// pub struct InsertionPoint {
//     block: BlockRef,
//     index: usize,
// }
//
// pub struct OpBuilder {
//     context: ContextRef,
//     insertion_point: InsertionPoint,
// }
//
// impl OpBuilder {
//     pub fn new(context: ContextRef, block: BlockRef) -> Rc<RefCell<Self>> {
//         let insertion_point = InsertionPoint { block, index: 0 };
//
//         Rc::new(RefCell::new(OpBuilder {
//             context,
//             insertion_point,
//         }))
//     }
//
//     pub fn insert(&mut self, operation: Operation) {
//         self.insertion_point
//             .block
//             .borrow_mut()
//             .operations
//             .insert(self.insertion_point.index, operation);
//         self.insertion_point.index += 1;
//     }
//
//     pub fn get_context(&self) -> ContextRef {
//         self.context.clone()
//     }
//
//     pub fn set_insertion_point_to_start(&mut self, block: BlockRef) {
//         self.insertion_point.block = block;
//         self.insertion_point.index = 0;
//     }
// }
