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

impl<S> VModel for PackedSlot<S>
where
    S: VModel,
{
    type Storage = S::Storage;
    const EXECUTE_POINTS: usize = 0;

    fn create(self, dp: &mut DirtyPoints, ctx: &EMCreateCtx) -> Self::Storage {
        self.slot.create(&mut patch_cursor(dp, self.dp_cursor), ctx)
    }

    fn update(self, storage: &mut Self::Storage, dp: &mut DirtyPoints, ctx: &EMCreateCtx) {
        self.slot
            .update(storage, &mut patch_cursor(dp, self.dp_cursor), ctx);
    }
}

fn patch_cursor<'r>(old: &'r mut DirtyPoints, cursor: Cursor) -> DirtyPoints<'r> {
    DirtyPoints {
        cursor,
        data: old.data,
    }
}
