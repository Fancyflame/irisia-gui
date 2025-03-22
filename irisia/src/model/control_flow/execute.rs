use crate::{
    model::tools::{caller_stack, DirtyPoints},
    prim_element::EMCreateCtx,
};

use super::VModel;

pub struct Execute<F> {
    updator: F,
}

impl<F, R> Execute<F>
where
    F: FnOnce() -> R,
    R: VModel,
{
    pub fn new(u: F) -> Self {
        Execute { updator: u }
    }
}

impl<F, R> VModel for Execute<F>
where
    F: FnOnce() -> R,
    R: VModel,
{
    const EXECUTE_POINTS: usize = 1 + R::EXECUTE_POINTS;
    type Storage = R::Storage;

    fn create(self, dp: &mut DirtyPoints, ctx: &EMCreateCtx) -> Self::Storage {
        let result = caller_stack::with_caller(dp.offset(), self.updator);
        dp.consume(1);
        result.create(dp, ctx)
    }

    fn update(self, storage: &mut Self::Storage, dp: &mut DirtyPoints, ctx: &EMCreateCtx) {
        let is_dirty = dp.check_range(Self::EXECUTE_POINTS);
        if !is_dirty {
            dp.consume(Self::EXECUTE_POINTS);
            return;
        }

        let vmodel = caller_stack::with_caller(dp.offset(), self.updator);
        dp.consume(1);
        vmodel.update(storage, dp, ctx);
    }
}
