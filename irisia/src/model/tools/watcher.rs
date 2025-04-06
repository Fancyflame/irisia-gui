use crate::model::tools::{caller_stack, DirtySet};
use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
};

use super::dependent_grid::DependentGrid;

pub struct Watcher<'a, T: ?Sized> {
    pub(crate) dep_grid: &'a DependentGrid,
    pub(crate) exec_point_id: usize,
    pub(crate) value: T,
}

trait MarkDirtySet {
    fn mark(&self, pos: usize);
}

impl<const DCAP: usize> MarkDirtySet for Cell<DirtySet<DCAP>> {
    fn mark(&self, pos: usize) {
        let mut ds = self.get();
        ds.mark(pos);
        self.set(ds);
    }
}

impl<T: ?Sized> Watcher<'_, T> {
    fn mark_dirty(&self) {
        if let Some(pos) = caller_stack::get_caller() {
            self.dep_grid.mark(self.exec_point_id, pos)
        }
    }
}

impl<T: ?Sized> Deref for Watcher<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.mark_dirty();
        &self.value
    }
}

impl<T: ?Sized> DerefMut for Watcher<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.mark_dirty();
        &mut self.value
    }
}
