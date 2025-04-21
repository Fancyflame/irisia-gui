use crate::{coerce_hook, hook::Signal};
use std::rc::Rc;

pub use self::common_vmodel::CommonVModel;

use super::{GetParentPropsFn, ModelCreateCtx, VModel};

pub mod branch;
pub mod common_vmodel;
pub mod miscellaneous;
pub mod repeat;
pub mod signal;

impl<T, Pp> From<Signal<T>> for Signal<dyn CommonVModel<CommonParentProps = Pp>>
where
    T: VModel<ParentProps = Pp> + 'static,
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
                type ParentProps = T::ParentProps;

                fn get_parent_props(&self, f: GetParentPropsFn<Self::ParentProps>) {
                    (**self).get_parent_props(f)
                }

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
