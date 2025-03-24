use std::marker::PhantomData;

use crate::{
    model::{tools::DirtyPoints, VModel},
    prim_element::EMCreateCtx,
};

pub use self::define_slot::{MergedPackedSlot, PackedSlot};

mod define_field;
mod define_slot;

pub struct VModelBuilder<T, F> {
    _default_src: PhantomData<T>,
    definitions: F,
}

impl<T> VModelBuilder<T, ()>
where
    T: Default,
{
    pub fn new() -> Self {
        Self {
            _default_src: PhantomData,
            definitions: (),
        }
    }
}

impl<T, F> VModelBuilder<T, F> {
    pub fn def_field<F2>(
        self,
        field_definer: F2,
    ) -> VModelBuilder<T, (F, define_field::DefineField<F2>)>
    where
        F2: FnOnce(&mut T),
    {
        VModelBuilder {
            _default_src: self._default_src,
            definitions: (self.definitions, define_field::DefineField(field_definer)),
        }
    }

    pub fn def_slot<S, F2>(
        self,
        slot: S,
        applicator: F2,
    ) -> VModelBuilder<T, (F, define_slot::DefineSlot<S, F2>)> {
        VModelBuilder {
            _default_src: self._default_src,
            definitions: (
                self.definitions,
                define_slot::DefineSlot { slot, applicator },
            ),
        }
    }
}

impl<T, F> VModel for VModelBuilder<T, F>
where
    T: Default + VModel,
    F: VModelBuilderNode<T>,
{
    // In most cases and from design principle, `T` should have no execute points.
    const EXECUTE_POINTS: usize = T::EXECUTE_POINTS + F::EXECUTE_POINTS;

    type Storage = T::Storage;

    fn create(self, dp: &mut DirtyPoints, ctx: &EMCreateCtx) -> Self::Storage {
        let mut src = T::default();
        self.definitions.create_build(&mut src, dp);
        src.create(dp, ctx)
    }

    fn update(self, storage: &mut Self::Storage, dp: &mut DirtyPoints, ctx: &EMCreateCtx) {
        let mut src = T::default();
        self.definitions.update_build(&mut src, dp);
        src.update(storage, dp, ctx);
    }
}

pub trait VModelBuilderNode<T> {
    const EXECUTE_POINTS: usize;

    fn create_build(self, src: &mut T, dp: &mut DirtyPoints);
    fn update_build(self, src: &mut T, dp: &mut DirtyPoints);
}

impl<T, F1, F2> VModelBuilderNode<T> for (F1, F2)
where
    F1: VModelBuilderNode<T>,
    F2: VModelBuilderNode<T>,
{
    const EXECUTE_POINTS: usize = F1::EXECUTE_POINTS + F2::EXECUTE_POINTS;

    fn create_build(self, src: &mut T, dp: &mut DirtyPoints) {
        self.0.create_build(src, dp);
        self.1.create_build(src, dp);
    }

    fn update_build(self, src: &mut T, dp: &mut DirtyPoints) {
        self.0.update_build(src, dp);
        self.1.update_build(src, dp);
    }
}

impl<T> VModelBuilderNode<T> for () {
    const EXECUTE_POINTS: usize = 0;
    fn create_build(self, _src: &mut T, _dp: &mut DirtyPoints) {}
    fn update_build(self, _src: &mut T, _dp: &mut DirtyPoints) {}
}
