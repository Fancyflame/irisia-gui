use std::{cell::RefCell, iter::Peekable};

pub use control_flow::{Model, VModel, VNode};
use dirty_set::DirtySetIter;

pub mod component;
pub mod control_flow;
mod dirty_set;

thread_local! {
    static UPDATE_POINT_STACK: RefCell<Vec<usize>> = RefCell::new(Vec::new());
}

struct UpdatePoints<'r, 's> {
    iter: &'r mut Peekable<DirtySetIter<'s>>,
    offset: usize,
}

impl<'s> UpdatePoints<'_, 's> {
    pub fn peek(&mut self) -> Option<usize> {
        self.iter.peek().map(|pos| *pos - self.offset)
    }

    pub fn step(&mut self) {
        self.iter.next();
    }

    pub fn nest<'r2>(&'r2 mut self, next_offset: usize) -> UpdatePoints<'r2, 's> {
        UpdatePoints {
            iter: self.iter,
            offset: self.offset + next_offset,
        }
    }
}
