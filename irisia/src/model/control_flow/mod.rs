use crate::{coerce_hook, hook::Signal};
use std::rc::Rc;

pub use self::common_vmodel::CommonVModel;

use super::{GetParentProps, ModelCreateCtx, VModel};

pub mod branch;
pub mod common_vmodel;
pub mod repeat;
pub mod signal;
pub mod tuple;

impl<T, Pp> From<Signal<T>> for Signal<dyn CommonVModel<Pp>>
where
    T: VModel + GetParentProps<Pp> + 'static,
{
    fn from(value: Signal<T>) -> Self {
        coerce_hook!(value)
    }
}

macro_rules! impl_vmodel_for_refs {
    ($($T:ty),*) => {
        $(
            impl<Pp, T> GetParentProps<Pp> for $T
            where
                T: GetParentProps<Pp>,
            {
                fn get_parent_props(&self, dst: &mut Vec<Pp>) {
                    (**self).get_parent_props(dst)
                }
            }

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
