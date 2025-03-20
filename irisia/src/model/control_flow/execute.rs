use crate::{
    model::optional_update::{caller_stack, DirtyPoints},
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

    fn create(self, exec_point_offset: usize, ctx: &EMCreateCtx) -> Self::Storage {
        caller_stack::with_caller(exec_point_offset, self.updator)
            .create(exec_point_offset + 1, ctx)
    }

    fn update(self, storage: &mut Self::Storage, mut dp: DirtyPoints, ctx: &EMCreateCtx) {
        let is_dirty = dp.check_range(Self::EXECUTE_POINTS);

        if is_dirty {
            let vmodel = caller_stack::with_caller(dp.offset(), self.updator);
            dp.consume(1);
            vmodel.update(storage, dp.nested(1), ctx);
        }
    }
}
