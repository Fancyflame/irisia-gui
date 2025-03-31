use std::cell::Cell;

use crate::{
    model::{
        tools::{caller_stack, DirtyPoints, DirtySet},
        VModel,
    },
    prim_element::EMCreateCtx,
};

pub struct Execute<F> {
    updator: F,
}

pub fn execute<F>(f: F) -> Execute<F>
where
    Execute<F>: VModel,
{
    Execute { updator: f }
}

impl<F, R> VModel for Execute<F>
where
    F: FnOnce() -> R,
    R: VModel,
{
    const EXECUTE_POINTS: usize = 1 + R::EXECUTE_POINTS;
    type Storage = R::Storage;

    fn create(self, dp: &mut DirtyPoints, ctx: &EMCreateCtx) -> Self::Storage {
        let result = caller_stack::with_caller(dp.offset(), || (self.updator)().create(dp, ctx));
        dp.consume(1);
        result
    }

    fn update(self, storage: &mut Self::Storage, dp: &mut DirtyPoints<DCAP>, ctx: &EMCreateCtx) {
        let is_dirty = dp.check_range(Self::EXECUTE_POINTS);
        if !is_dirty {
            dp.consume(Self::EXECUTE_POINTS);
            return;
        }

        caller_stack::with_caller(dp.offset(), || (self.updator)().update(storage, dp, ctx));
        dp.consume(1);
    }
}
