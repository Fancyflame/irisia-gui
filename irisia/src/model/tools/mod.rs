pub(crate) use dirty_set::DirtySet;
use dirty_set::DirtySetIter;
use std::ops::{Deref, DerefMut};

pub(crate) mod caller_stack;
mod dirty_set;

pub struct DirtyPoints<'a> {
    iter: DirtySetIter<'a>,
    offset: usize,
}

impl<'a> DirtyPoints<'a> {
    pub(crate) fn new(iter: DirtySetIter<'a>) -> Self {
        Self { iter, offset: 0 }
    }

    pub fn check_range(&self, upper_bound: usize) -> bool {
        match self.iter.peek() {
            Some(p) => p < self.offset + upper_bound,
            None => false,
        }
    }

    pub fn consume(&mut self, upper_bound: usize) {
        self.offset += upper_bound;
        self.iter.set_cursor(self.offset);
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn fork(&self) -> Self {
        DirtyPoints {
            iter: self.iter.clone(),
            offset: self.offset,
        }
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
