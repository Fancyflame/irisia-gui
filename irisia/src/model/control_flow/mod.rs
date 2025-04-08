use crate::{coerce_hook, hook::Signal, prim_element::EMCreateCtx};
use std::rc::Rc;

pub use self::common_vmodel::CommonVModel;

use super::VModel;

pub mod branch;
pub mod common_vmodel;
pub mod repeat;
pub mod signal;
pub mod tuple;

impl<T> From<Signal<T>> for Signal<dyn CommonVModel>
where
    T: VModel + 'static,
{
    fn from(value: Signal<T>) -> Self {
        coerce_hook!(value)
    }
}

macro_rules! impl_vmodel_for_refs {
    ($($T:ty),*) => {
        $(
            impl<T> VModel for $T
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
        )*
    };
}

impl_vmodel_for_refs!(Box<T>, Rc<T>, &T);
