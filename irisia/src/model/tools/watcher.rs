use crate::model::tools::{caller_stack, DirtySet};
use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
};

use super::dependent_grid::DependentGrid;

pub struct Watcher<'a, T: ?Sized> {
    dep: DepType<'a>,
    value: T,
}

enum DepType<'a> {
    FieldHook(&'a dyn MarkDirtySet),
    TempVar {
        dep_grid: &'a DependentGrid,
        exec_point_id: usize,
    },
}

impl<'a, T> Watcher<'a, T> {
    pub(crate) fn from_field_hook<const DCAP: usize>(
        dp: &'a Cell<DirtySet<DCAP>>,
        value: T,
    ) -> Self {
        Self {
            value,
            dep: DepType::FieldHook(dp),
        }
    }

    pub(crate) fn from_temp_var(
        dep_grid: &'a DependentGrid,
        exec_point_id: usize,
        value: T,
    ) -> Self {
        Self {
            value,
            dep: DepType::TempVar {
                dep_grid,
                exec_point_id,
            },
        }
    }
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
            match self.dep {
                DepType::FieldHook(h) => h.mark(pos),
                DepType::TempVar {
                    dep_grid,
                    exec_point_id,
                } => dep_grid.mark(exec_point_id, pos),
            }
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
