use crate::model::tools::caller_stack;
use std::ops::{Deref, DerefMut};

use super::{dependent_grid::DependentGrid, field_deps::FieldDeps};

pub struct Watcher<'a, T: ?Sized> {
    pub(super) wt: WatcherType<'a>,
    pub(super) value: T,
}

pub(super) enum WatcherType<'a> {
    Field {
        field_deps: &'a FieldDeps,
        id: usize,
    },
    TempVar {
        grid: &'a DependentGrid,
        exec_point: usize,
    },
}

impl<T: ?Sized> Watcher<'_, T> {
    fn mark_dirty(&self) {
        if let Some(pos) = caller_stack::get_caller() {
            match self.wt {
                WatcherType::Field { field_deps, id } => field_deps.mark(id, pos),
                WatcherType::TempVar {
                    grid,
                    exec_point: exec_point_id,
                } => grid.mark(exec_point_id, pos),
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
