use crate::{
    model::{tools::DirtyPoints, VModel},
    prim_element::EMCreateCtx,
};

pub use self::define_slot::PackedSlot;

mod define_field;
mod define_slot;

pub struct VModelBuilder<T, D> {
    create_blank_prop: fn() -> T,
    definitions: D,
}

impl<T> VModelBuilder<T, ()> {
    pub fn new(f: fn() -> T) -> Self {
        Self {
            create_blank_prop: f,
            definitions: (),
        }
    }
}

impl<T, D> VModelBuilder<T, D> {
    pub fn def_field<F2>(
        self,
        field_definer: F2,
    ) -> VModelBuilder<T, (D, define_field::DefineField<F2>)>
    where
        F2: FnOnce(&mut T),
    {
        VModelBuilder {
            create_blank_prop: self.create_blank_prop,
            definitions: (self.definitions, define_field::DefineField(field_definer)),
        }
    }

    pub fn def_slot<S, F2>(
        self,
        applicator: F2,
        slot: S,
    ) -> VModelBuilder<T, (D, define_slot::DefineSlot<S, F2>)>
    where
        F2: FnOnce(&mut T, PackedSlot<S>),
    {
        VModelBuilder {
            create_blank_prop: self.create_blank_prop,
            definitions: (
                self.definitions,
                define_slot::DefineSlot { slot, applicator },
            ),
        }
    }
}

impl<'a, T, F> VModel<'a> for VModelBuilder<T, F>
where
    T: Default + VModel<'a>,
    F: VModelBuilderNode<'a, T>,
{
    // In most cases and from design principle, `T` should have no execute points.
    const EXECUTE_POINTS: usize = T::EXECUTE_POINTS + F::EXECUTE_POINTS;

    type Storage = T::Storage;

    fn create(self, dp: &mut DirtyPoints<'a>, ctx: &EMCreateCtx) -> Self::Storage {
        let mut src = (self.create_blank_prop)();
        self.definitions.create_build(&mut src, dp);
        src.create(dp, ctx)
    }

    fn update(self, storage: &mut Self::Storage, dp: &mut DirtyPoints<'a>, ctx: &EMCreateCtx) {
        let mut src = (self.create_blank_prop)();
        self.definitions.update_build(&mut src, dp);
        src.update(storage, dp, ctx);
    }
}

pub trait VModelBuilderNode<'a, T> {
    const EXECUTE_POINTS: usize;

    fn create_build(self, src: &mut T, dp: &mut DirtyPoints<'a>);
    fn update_build(self, src: &mut T, dp: &mut DirtyPoints<'a>);
}

impl<'a, T, F1, F2> VModelBuilderNode<'a, T> for (F1, F2)
where
    F1: VModelBuilderNode<'a, T>,
    F2: VModelBuilderNode<'a, T>,
{
    const EXECUTE_POINTS: usize = F1::EXECUTE_POINTS + F2::EXECUTE_POINTS;

    fn create_build(self, src: &mut T, dp: &mut DirtyPoints<'a>) {
        self.0.create_build(src, dp);
        self.1.create_build(src, dp);
    }

    fn update_build(self, src: &mut T, dp: &mut DirtyPoints<'a>) {
        self.0.update_build(src, dp);
        self.1.update_build(src, dp);
    }
}

impl<T> VModelBuilderNode<'_, T> for () {
    const EXECUTE_POINTS: usize = 0;
    fn create_build(self, _src: &mut T, _dp: &mut DirtyPoints) {}
    fn update_build(self, _src: &mut T, _dp: &mut DirtyPoints) {}
}
