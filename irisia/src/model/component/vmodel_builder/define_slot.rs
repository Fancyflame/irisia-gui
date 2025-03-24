use crate::{
    model::{
        tools::{Cursor, DirtyPoints},
        VModel,
    },
    prim_element::EMCreateCtx,
};

use super::VModelBuilderNode;

pub struct DefineSlot<S, F> {
    pub(super) slot: S,
    pub(super) applicator: F,
}

impl<T, S, F> VModelBuilderNode<T> for DefineSlot<S, F>
where
    S: VModel,
    F: Fn(&mut T, PackedSlot<S>),
{
    const EXECUTE_POINTS: usize = S::EXECUTE_POINTS;

    fn create_build(self, src: &mut T, dp: &mut DirtyPoints) {
        (self.applicator)(
            src,
            PackedSlot {
                dp_cursor: dp.cursor.clone(),
                slot: self.slot,
            },
        );
        dp.consume(S::EXECUTE_POINTS);
    }

    fn update_build(self, src: &mut T, dp: &mut DirtyPoints) {
        self.create_build(src, dp);
    }
}

pub struct PackedSlot<S> {
    slot: S,
    dp_cursor: Cursor,
}

impl<S> PackedSlot<S> {
    pub fn merge<'a>(self, dp: &DirtyPoints<'a>) -> MergedPackedSlot<'a, S> {
        MergedPackedSlot {
            slot: self.slot,
            dp: DirtyPoints {
                cursor: self.dp_cursor,
                data: dp.data,
            },
        }
    }
}

pub struct MergedPackedSlot<'a, S> {
    slot: S,
    dp: DirtyPoints<'a>,
}

impl<S> VModel for MergedPackedSlot<'_, S>
where
    S: VModel,
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
