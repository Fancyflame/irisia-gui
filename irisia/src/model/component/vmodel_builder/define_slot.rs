use crate::{
    model::{tools::DirtyPoints, VModel},
    prim_element::EMCreateCtx,
};

use super::VModelBuilderNode;

pub struct DefineSlot<S, F> {
    pub(crate) slot: S,
    pub(crate) applicator: F,
}

impl<'dp, T, S, F> VModelBuilderNode<'dp, T> for DefineSlot<S, F>
where
    S: VModel,
    F: Fn(&mut T, PackedSlot<'dp, S>),
{
    const EXECUTE_POINTS: usize = S::EXECUTE_POINTS;

    fn create_build(self, src: &mut T, exec_point_offset: usize) {
        (self.applicator)(
            src,
            PackedSlot {
                meta: PackedSlotMeta::ExecPointOffset(exec_point_offset),
                slot: self.slot,
            },
        );
    }

    fn update_build(self, src: &mut T, mut dp: DirtyPoints<'_, 'dp>) {
        let static_dp = dp.fork();
        dp.consume(S::EXECUTE_POINTS);

        (self.applicator)(
            src,
            PackedSlot {
                meta: PackedSlotMeta::DirtyPoints(static_dp),
                slot: self.slot,
            },
        );
    }
}

pub(crate) enum PackedSlotMeta<'a> {
    ExecPointOffset(usize),
    DirtyPoints(DirtyPoints<'a, 'a>),
}

pub struct PackedSlot<'a, S> {
    meta: PackedSlotMeta<'a>,
    slot: S,
}

impl<S> VModel for PackedSlot<'_, S>
where
    S: VModel,
{
    type Storage = S::Storage;
    const EXECUTE_POINTS: usize = 0;

    fn create(self, _: usize, ctx: &EMCreateCtx) -> Self::Storage {
        match self.meta {
            PackedSlotMeta::ExecPointOffset(offset) => self.slot.create(offset, ctx),
            PackedSlotMeta::DirtyPoints(dp) => self.slot.create(dp.offset(), ctx),
        }
    }

    fn update(self, storage: &mut Self::Storage, _: DirtyPoints, ctx: &EMCreateCtx) {
        match self.meta {
            PackedSlotMeta::ExecPointOffset(_) => panic!("should be meta for updating"),
            PackedSlotMeta::DirtyPoints(dp) => self.slot.update(storage, dp, ctx),
        }
    }
}
