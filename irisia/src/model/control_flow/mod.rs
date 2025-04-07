use crate::{coerce_hook, hook::Signal, prim_element::EMCreateCtx};
use std::rc::Rc;

pub use self::common_vmodel::CommonVModel;

use super::VModel;

pub mod branch;
pub mod common_vmodel;
pub mod repeat;
pub mod signal;
pub mod tuple;

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

impl<T> VModel for Rc<T>
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

impl<T> From<Signal<T>> for Signal<dyn CommonVModel>
where
    T: VModel + 'static,
{
    fn from(value: Signal<T>) -> Self {
        coerce_hook!(value)
    }
}
