use dependent_grid::DependentGrid;
use dirty_set::bitset_create;
use field_deps::FieldDeps;
use iter::DirtyPoints;

use super::VModel;

pub(crate) mod caller_stack;
pub mod cursor;
mod dependent_grid;
mod dirty_set;
mod field_deps;
pub mod iter;
pub mod watcher;

pub struct DepManager {
    current_dp: Box<[u8]>,
    field_deps: FieldDeps,
    grid: DependentGrid,
}

impl DepManager {
    pub fn new<'a, T>(field_count: usize, _vmodel: &T) -> Self
    where
        T: VModel<'a>,
    {
        let width = T::EXECUTE_POINTS;
        Self {
            current_dp: bitset_create(width),
            field_deps: FieldDeps::new(field_count, width),
            grid: DependentGrid::new(width),
        }
    }

    pub fn iter_builder(&mut self) -> DpBuilder {
        self.current_dp.fill(0);
        DpBuilder { mgr: self }
    }
}

pub struct DpBuilder<'a> {
    mgr: &'a mut DepManager,
}

impl<'a> DpBuilder<'a> {
    pub fn set_updated_field(&mut self, id: usize) {
        self.mgr.field_deps.take(&mut self.mgr.current_dp, id);
    }

    pub fn build(self) -> DirtyPoints<'a> {
        DirtyPoints::new(self.mgr)
    }
}
