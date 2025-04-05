use crate::{hook::Signal, prim_element::EMCreateCtx};

pub use self::branch::Branch;

use super::VModel;

pub mod branch;
pub mod common_vmodel;
pub mod repeat;
pub mod tuple;

impl<T> VModel for Signal<T>
where
    T: VModel + ?Sized,
{
    type Storage = T::Storage;

    fn create(&self, ctx: &EMCreateCtx) -> Self::Storage {
        self.read().create(ctx)
    }

    fn update(&self, storage: &mut Self::Storage, ctx: &EMCreateCtx) {
        self.read().update(storage, ctx);
    }
}

impl<T> VModel for Box<T>
where
    T: VModel + ?Sized,
{
    type Storage = T::Storage;

    fn create(&self, ctx: &EMCreateCtx) -> Self::Storage {
        (**self).create(ctx)
    }

    fn update(&self, storage: &mut Self::Storage, ctx: &EMCreateCtx) {
        (**self).update(storage, ctx);
    }
}
