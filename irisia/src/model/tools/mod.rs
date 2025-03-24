pub(crate) use dirty_set::{Cursor, DirtySet};
use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

pub(crate) mod caller_stack;
mod dirty_set;

pub struct DirtyPointsSrc<const D_CAP: usize>(RefCell<[u8; D_CAP]>);

impl<const D_CAP: usize> DirtyPointsSrc<D_CAP> {
    pub fn new(ds: DirtySet<D_CAP>) -> Self {
        Self(RefCell::new(ds.data()))
    }

    pub fn to_dp(&self) -> DirtyPoints {
        DirtyPoints::new(&self.0)
    }
}

pub struct DirtyPoints<'a> {
    pub(crate) cursor: Cursor,
    pub(crate) data: &'a RefCell<[u8]>,
}

impl<'a> DirtyPoints<'a> {
    fn new(data: &'a RefCell<[u8]>) -> Self {
        Self {
            cursor: Cursor::new(0),
            data,
        }
    }

    pub fn check_range(&self, upper_bound: usize) -> bool {
        let peeked = self.cursor.clone().next(&self.data.borrow());
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

    pub fn fork(&self) -> DirtyPoints {
        DirtyPoints {
            cursor: self.cursor.clone(),
            data: self.data,
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
