use std::ops::{Deref, DerefMut};

use crate::hook::utils::trace_cell::TraceMut;

use super::inner::Inner;

pub struct ReactiveWriteGuard<'a, T> {
    r: TraceMut<'a, T>,
    _drop: DropGuard<'a, T>,
}

impl<'a, T> ReactiveWriteGuard<'a, T> {
    pub(super) fn new(r: TraceMut<'a, T>, inner: &'a Inner<T>) -> Self {
        Self {
            r,
            _drop: DropGuard(inner),
        }
    }
}

impl<T> Deref for ReactiveWriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.r
    }
}

impl<T> DerefMut for ReactiveWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.r
    }
}

struct DropGuard<'a, T>(&'a Inner<T>);

impl<T> Drop for DropGuard<'_, T> {
    fn drop(&mut self) {
        self.0.recall_delayed_callback();
    }
}
