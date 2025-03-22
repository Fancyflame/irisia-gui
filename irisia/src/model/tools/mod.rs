pub(crate) use dirty_set::DirtySet;
use dirty_set::DirtySetIter;
use std::{
    iter::Peekable,
    ops::{Deref, DerefMut},
};

pub(crate) mod caller_stack;
mod dirty_set;

pub struct DirtyPoints<'i, 'a> {
    iter: MaybeOwned<'i, Peekable<DirtySetIter<'a>>>,
    offset: usize,
}

impl<'a> DirtyPoints<'_, 'a> {
    pub(crate) fn new(iter: DirtySetIter<'a>) -> Self {
        Self {
            iter: MaybeOwned::Owned(iter.peekable()),
            offset: 0,
        }
    }

    pub fn check_range(&mut self, upper_bound: usize) -> bool {
        match self.iter.peek() {
            Some(p) => (self.offset..self.offset + upper_bound).contains(p),
            None => false,
        }
    }

    pub fn consume(&mut self, upper_bound: usize) {
        loop {
            self.iter.next_if(|&p| p < self.offset + upper_bound);
        }
    }

    pub fn nested(&mut self, offset: usize) -> DirtyPoints<'_, 'a> {
        let next = self.iter.peek().copied();
        let new_offset = self.offset + offset;

        debug_assert!(
            !matches!(next, Some(n) if n < new_offset),
            "invalid offset: there still points not consumed before offset"
        );

        DirtyPoints {
            iter: MaybeOwned::RefMut(&mut *self.iter),
            offset: new_offset,
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn fork<'r>(&self) -> DirtyPoints<'r, 'a> {
        DirtyPoints {
            iter: MaybeOwned::Owned(self.iter.clone()),
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
