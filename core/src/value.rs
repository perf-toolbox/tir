use std::marker::PhantomData;
use std::sync::Arc;

use crate::AllocId;
use crate::BlockArg;
use crate::BlockWRef;
use crate::ContextRef;
use crate::ContextWRef;
use crate::OpRef;
use crate::Type;

#[derive(Debug, Clone)]
pub enum ValueOwner {
    Op(AllocId),
    BlockArg(BlockArg),
}

#[derive(Debug)]
pub struct Value<T: Into<Type> + TryFrom<Type> = Type> {
    owner: ValueOwner,
    context: ContextWRef,
    name: Arc<String>,
    _a: PhantomData<T>,
}

impl<T: Into<Type> + TryFrom<Type> + Clone> Value<T> {
    pub fn from_block_arg(context: ContextWRef, name: &str, arg: BlockArg) -> Self {
        Value{
            owner: ValueOwner::BlockArg(arg),
            context,
            name: Arc::new(name.to_string()),
            _a: PhantomData::default(),
        }
    }

    pub fn get_context(&self) -> ContextRef {
        self.context.upgrade().unwrap()
    }

    // pub fn get_defining_op(&self) -> OpRef {
    //     self.get_context().get_op(self.op_id).unwrap()
    // }

    pub fn get_type(&self) -> T {
        match &self.owner {
            ValueOwner::BlockArg(arg) => { todo!() },
            _ => unimplemented!(),
        }
    }
}

// pub type AnyValue = Value<Type>;

// #[derive(Debug, PartialEq, Clone)]
// pub struct AnyValue {
//     pub op_id: AllocId,
//     pub ty: Type,
// }

// During code parsing, Type cannot be known statically, so AnyValue is used to represent a value without knowing
// its type at compile time, while Value<T> is used to represent a value with a statically known type.
// This allows us to convert an AnyValue to a Value<T> when we know the type of the value at compile time, and vice versa.
// impl AnyValue {
//     pub fn get_context(&self) -> ContextRef {
//         self.get_type().get_context().unwrap()
//     }

//     pub fn get_defining_op(&self) -> OpRef {
//         self.get_context().get_op(self.op_id).unwrap()
//     }

//     pub fn get_type(&self) -> Type {
//         self.ty.clone()
//     }
// }

// impl<T: Into<Type> + TryFrom<Type>> From<Value<T>> for AnyValue {
//     fn from(value: Value<T>) -> Self {
//         AnyValue {
//             op_id: value.op_id,
//             ty: value.ty.into(),
//         }
//     }
// }

// impl<T: Into<Type> + TryFrom<Type>> TryInto<Value<T>> for AnyValue {
//     type Error = ();

//     fn try_into(self) -> Result<Value<T>, Self::Error> {
//         let ty: T = self.ty.try_into().map_err(|_| ())?;
//         Ok(Value {
//             op_id: self.op_id,
//             ty,
//         })
//     }
// }
