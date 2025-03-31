use crate::{
    model::{tools::DirtyPoints, VModel},
    prim_element::EMCreateCtx,
};

use super::VModelBuilderNode;

pub struct DefineSlot<S, F> {
    pub(super) slot: S,
    pub(super) applicator: F,
}

impl<'a, T, S, F> VModelBuilderNode<'a, T> for DefineSlot<S, F>
where
    S: VModel<'a>,
    F: Fn(&mut T, PackedSlot<'a, S>),
{
    const EXECUTE_POINTS: usize = S::EXECUTE_POINTS;

    fn create_build(self, src: &mut T, dp: &mut DirtyPoints<'a>) {
        (self.applicator)(
            src,
            PackedSlot {
                dp: dp.fork(),
                slot: self.slot,
            },
        );
        dp.consume(S::EXECUTE_POINTS);
    }

    fn update_build(self, src: &mut T, dp: &mut DirtyPoints<'a>) {
        self.create_build(src, dp);
    }
}

pub struct PackedSlot<'a, S> {
    slot: S,
    dp: DirtyPoints<'a>,
}

impl<'a, S> VModel<'_> for PackedSlot<'a, S>
where
    S: VModel<'a>,
{
    type Storage = S::Storage;
    const EXECUTE_POINTS: usize = 0;

    fn create(mut self, _: &mut DirtyPoints, ctx: &EMCreateCtx) -> Self::Storage {
        self.slot.create(&mut self.dp, ctx)
    }

    fn update(mut self, storage: &mut Self::Storage, _: &mut DirtyPoints, ctx: &EMCreateCtx) {
        self.slot.update(storage, &mut self.dp, ctx);
    }
}
