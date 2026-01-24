use std::rc::Rc;

use super::{ModelCreateCtx, VModel};

pub mod branch;
pub mod general;
pub mod repeat;
pub mod signal;
pub mod tuple;

macro_rules! impl_vmodel_for_refs {
    ($($T:ty),*) => {
        $(
            impl<T> VModel for $T
            where
                T: VModel + ?Sized,
            {
                type Storage = T::Storage;

                fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
                    (**self).create(ctx)
                }

                fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx) {
                    (**self).update(storage, ctx);
                }
            }
        )*
    };
}

impl_vmodel_for_refs!(Box<T>, Rc<T>, &T);
