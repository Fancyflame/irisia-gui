use std::ops::{Deref, DerefMut};

use crate::hook::{listener::CallbackAction, trace_cell::TraceMut, utils::ListenerList};

pub struct WriteGuard<'a, T: ?Sized> {
    // do not swap the field order
    r: TraceMut<'a, T>,
    _wl: WakeListeners<'a>,
}

impl<'a, T: ?Sized> WriteGuard<'a, T> {
    pub(super) fn new(r: TraceMut<'a, T>, ll: &'a ListenerList) -> Self {
        Self {
            r,
            _wl: WakeListeners(ll),
        }
    }
}

impl<T: ?Sized> Deref for WriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.r
    }
}

impl<T: ?Sized> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.r
    }
}

struct WakeListeners<'a>(&'a ListenerList);

impl Drop for WakeListeners<'_> {
    fn drop(&mut self) {
        self.0.callback_all(CallbackAction::RegisterDirty);
        self.0.callback_all(CallbackAction::Update);
    }
}
