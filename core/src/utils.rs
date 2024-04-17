use std::any::TypeId;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TraitId(TypeId);

pub fn trait_id<T: ?Sized + 'static>() -> TraitId {
    TraitId(TypeId::of::<T>())
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
