use std::ops::{Deref, DerefMut};

use crate::hook::utils::trace_cell::TraceMut;

use super::inner::Inner;

pub struct ReactiveWriteGuard<'a, T: ?Sized> {
    pub(super) r: TraceMut<'a, T>,
    pub(super) inner: &'a Inner<T>,
}

impl<T: ?Sized> Drop for ReactiveWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.inner.recall_delayed_callback(&mut self.r);
    }
}

impl<T: ?Sized> Deref for ReactiveWriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.r
    }
}

impl<T: ?Sized> DerefMut for ReactiveWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.r
    }
}
