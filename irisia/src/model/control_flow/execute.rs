use crate::{
    model::tools::{caller_stack, dependent_grid::DependentGrid, watcher::Watcher, DirtyPoints},
    prim_element::EMCreateCtx,
};

use super::VModel;

pub struct Execute<F> {
    pub updator: F,
}

impl<'a, F, R> VModel<'a> for Execute<F>
where
    F: FnOnce(WatcherMaker<'a>) -> R,
    R: VModel<'a>,
{
    const EXECUTE_POINTS: usize = 1 + R::EXECUTE_POINTS;
    type Storage = R::Storage;

    fn create(self, dp: &mut DirtyPoints<'a>, ctx: &EMCreateCtx) -> Self::Storage {
        let wm = WatcherMaker {
            dep_grid: dp.dep_grid(),
            id: dp.offset(),
        };

        dp.consume(1);
        caller_stack::with_caller(dp.offset(), || (self.updator)(wm).create(dp, ctx))
    }

    fn update(self, storage: &mut Self::Storage, dp: &mut DirtyPoints<'a>, ctx: &EMCreateCtx) {
        let is_dirty = dp.check_range(Self::EXECUTE_POINTS);
        if !is_dirty {
            dp.consume(Self::EXECUTE_POINTS);
            return;
        }

        let wm = WatcherMaker {
            dep_grid: dp.dep_grid(),
            id: dp.offset(),
        };
        dp.consume(1);
        caller_stack::with_caller(dp.offset(), || (self.updator)(wm).update(storage, dp, ctx));
    }
}

pub struct WatcherMaker<'a> {
    dep_grid: &'a DependentGrid,
    id: usize,
}

impl<'a> WatcherMaker<'a> {
    pub fn make<T>(&self, value: T) -> Watcher<'a, T> {
        Watcher::from_temp_var(self.dep_grid, self.id, value)
    }
}
