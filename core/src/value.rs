use std::marker::PhantomData;
use std::sync::Arc;

use crate::AllocId;
use crate::BlockArg;
use crate::BlockRef;
use crate::ContextRef;
use crate::ContextWRef;
use crate::OpRef;
use crate::Ty;
use crate::Type;

#[derive(Debug, Clone)]
pub enum ValueOwner {
    Op(AllocId),
    BlockArg(BlockArg),
}

#[derive(Debug, Clone)]
pub struct Value<T: Into<Type> + TryFrom<Type> + Clone = Type> {
    owner: ValueOwner,
    context: ContextWRef,
    name: Arc<String>,
    _a: PhantomData<T>,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        let context = self.context.upgrade();
        let other_context = other.context.upgrade();

        if context.is_none() || other_context.is_none() {
            return false;
        }

        let context = context.unwrap();
        let other_context = other_context.unwrap();

        if Arc::as_ptr(&context) != Arc::as_ptr(&other_context) {
            return false;
        }

        match (&self.owner, &other.owner) {
            (ValueOwner::Op(id), ValueOwner::Op(other_id)) => id == other_id,
            (ValueOwner::BlockArg(arg), ValueOwner::BlockArg(other_arg)) => arg == other_arg,
            _ => false,
        }
    }
}

impl<T: Into<Type> + TryFrom<Type> + Clone> Value<T> {
    pub fn from_block_arg(context: ContextWRef, name: &str, arg: BlockArg) -> Self {
        Value {
            owner: ValueOwner::BlockArg(arg),
            context,
            name: Arc::new(name.to_string()),
            _a: PhantomData,
        }
    }

    pub fn from_op(context: ContextRef, name: &str, alloc_id: AllocId) -> Self {
        Value {
            owner: ValueOwner::Op(alloc_id),
            context: Arc::downgrade(&context),
            name: Arc::new(name.to_string()),
            _a: PhantomData,
        }
    }

    pub fn get_context(&self) -> ContextRef {
        self.context.upgrade().unwrap()
    }

    pub fn get_defining_op(&self) -> Option<OpRef> {
        match &self.owner {
            ValueOwner::Op(alloc_id) => {
                let context = self.context.upgrade().unwrap();
                context.get_op(*alloc_id)
            }
            _ => None,
        }
    }

    pub fn get_defining_block(&self) -> Option<BlockRef> {
        match &self.owner {
            ValueOwner::BlockArg(arg) => arg.get_block(),
            _ => None,
        }
    }

    pub fn get_type(&self) -> T {
        match &self.owner {
            ValueOwner::BlockArg(arg) => match arg.get_type().try_into() {
                Ok(res) => res,
                _ => unreachable!(),
            },
            ValueOwner::Op(alloc_id) => {
                let context = self.context.upgrade().unwrap();
                let op = context.get_op(*alloc_id).clone().unwrap();
                let ret_ty = op.borrow().get_return_type().unwrap();

                match ret_ty.try_into() {
                    Ok(res) => res,
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl Value<Type> {
    pub fn try_cast<Target: Ty + Clone + Into<Type> + TryFrom<Type>>(
        &self,
    ) -> Result<Value<Target>, ()> {
        let ty = self.get_type();

        if !ty.isa::<Target>() {
            return Err(());
        }

        Ok(Value {
            owner: self.owner.clone(),
            context: self.context.clone(),
            name: self.name.clone(),
            _a: PhantomData,
        })
    }
}

impl<T: Ty + Clone + Into<Type> + TryFrom<Type>> From<Value<T>> for Value {
    fn from(value: Value<T>) -> Self {
        Value {
            owner: value.owner,
            context: value.context,
            name: value.name,
            _a: PhantomData,
        }
    }
}
