use std::ops::{Deref, DerefMut};

use super::Common;

pub struct RedrawGuard<'a, T: ?Sized> {
    r: &'a mut T,
    common: &'a mut Common,
}

impl<'a, T: ?Sized> RedrawGuard<'a, T> {
    pub(super) fn new(r: &'a mut T, common: &'a mut Common) -> Self {
        Self { r, common }
    }
}

impl<T: ?Sized> Deref for RedrawGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.r
    }
}

impl<T: ?Sized> DerefMut for RedrawGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.r
    }
}

impl<T: ?Sized> Drop for RedrawGuard<'_, T> {
    fn drop(&mut self) {
        self.common.request_repaint();
    }
}
