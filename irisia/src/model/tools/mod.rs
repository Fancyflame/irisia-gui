use dependent_grid::DependentGrid;
pub(crate) use dirty_set::{Cursor, DirtySet};
use std::ops::{Deref, DerefMut};

pub(crate) mod caller_stack;
pub mod dependent_grid;
mod dirty_set;
pub mod watcher;

pub struct DirtyPoints<'a> {
    cursor: Cursor,
    data: &'a [u8],
    grid: &'a DependentGrid,
}

impl<'a> DirtyPoints<'a> {
    fn new(data: &'a [u8], dep_grid: &'a DependentGrid) -> Self {
        Self {
            cursor: Cursor::new(0),
            data,
            grid: dep_grid,
        }
    }

    pub fn check_range(&self, upper_bound: usize) -> bool {
        let peeked = self.cursor.clone().next(&self.data);
        match peeked {
            Some(p) => p < self.offset() + upper_bound,
            None => false,
        }
    }

    pub fn consume(&mut self, upper_bound: usize) {
        self.cursor = Cursor::new(self.cursor.offset() + upper_bound);
    }

    pub fn offset(&self) -> usize {
        self.cursor.offset()
    }

    pub(crate) fn fork(&self) -> DirtyPoints {
        DirtyPoints {
            cursor: self.cursor.clone(),
            data: self.data,
            grid: self.grid,
        }
    }

    pub(crate) fn dep_grid(&self) -> &DependentGrid {
        self.grid
    }
}

enum MaybeOwned<'a, T> {
    Owned(T),
    RefMut(&'a mut T),
}

impl<T> Deref for MaybeOwned<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(t) => t,
            Self::RefMut(t) => t,
        }
    }
}

impl<T> DerefMut for MaybeOwned<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Owned(t) => t,
            Self::RefMut(t) => t,
        }
    }
}

fn mark_bit(data: &mut [u8], position: usize) {
    let index = position / 8;
    let shifts = position % 8;
    let byte = data.get_mut(index).expect("position out of limit");
    *byte |= 1 << shifts;
}
