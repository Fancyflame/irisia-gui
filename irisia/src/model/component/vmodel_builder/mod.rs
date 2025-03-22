use std::marker::PhantomData;

use crate::{
    model::{tools::DirtyPoints, VModel},
    prim_element::EMCreateCtx,
};

mod define_field;
mod define_slot;

pub struct VModelBuilder<T, F> {
    _default_src: PhantomData<T>,
    field_definitions: F,
}

impl<T> VModelBuilder<T, ()>
where
    T: Default,
{
    pub fn new() -> Self {
        Self {
            _default_src: PhantomData,
            field_definitions: (),
        }
    }
}

impl<T, F> VModelBuilder<T, F> {
    pub fn def<F2>(self, field_definer: F2) -> VModelBuilder<T, (F, define_field::DefineField<F2>)>
    where
        F2: FnOnce(&mut T),
    {
        VModelBuilder {
            _default_src: self._default_src,
            field_definitions: (
                self.field_definitions,
                define_field::DefineField(field_definer),
            ),
        }
    }
}

impl<T, F> VModel for VModelBuilder<T, F>
where
    T: Default + VModel,
    F: VModelBuilderNode<T>,
{
    const EXECUTE_POINTS: usize = F::EXECUTE_POINTS + T::EXECUTE_POINTS;
    type Storage = T::Storage;

    fn create(self, exec_point_offset: usize, ctx: &EMCreateCtx) -> Self::Storage {
        let mut src = T::default();
        self.field_definitions
            .create_build(&mut src, exec_point_offset);
        src.create(exec_point_offset + F::EXECUTE_POINTS, ctx)
    }

    fn update(self, storage: &mut Self::Storage, mut dp: DirtyPoints, ctx: &EMCreateCtx) {
        let mut src = T::default();
        self.field_definitions.update_build(&mut src, dp.nested(0));
        src.update(storage, dp.nested(F::EXECUTE_POINTS), ctx);
    }
}

pub trait VModelBuilderNode<'dp, T> {
    const EXECUTE_POINTS: usize;

    fn create_build(self, src: &mut T, dp: &mut DirtyPoints<'dp>);
    fn update_build(self, src: &mut T, dp: &mut DirtyPoints<'dp>);
}

impl<'dp, T, F1, F2> VModelBuilderNode<'dp, T> for (F1, F2)
where
    F1: VModelBuilderNode<'dp, T>,
    F2: VModelBuilderNode<'dp, T>,
{
    const EXECUTE_POINTS: usize = F1::EXECUTE_POINTS + F2::EXECUTE_POINTS;

    fn create_build(self, src: &mut T, exec_point_offset: usize) {
        self.0.create_build(src, exec_point_offset);
        self.1
            .create_build(src, exec_point_offset + F1::EXECUTE_POINTS);
    }

    fn update_build(self, src: &mut T, mut dp: DirtyPoints<'_, 'dp>) {
        self.0.update_build(src, dp.nested(0));
        self.1.update_build(src, dp.nested(F1::EXECUTE_POINTS));
    }
}

impl<T> VModelBuilderNode<'_, T> for () {
    const EXECUTE_POINTS: usize = 0;
    fn create_build(self, _src: &mut T, _exec_point_offset: usize) {}
    fn update_build(self, _src: &mut T, _dp: DirtyPoints) {}
}
