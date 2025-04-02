use crate::hook::{
    listener::CallbackAction,
    utils::{trace_cell::TraceMut, ListenerList},
};
use std::ops::{Deref, DerefMut};

pub struct WriteGuard<'a, T: ?Sized> {
    r: TraceMut<'a, T>,    // let this to drop first
    wl: WakeListeners<'a>, // then update the listener list
}

impl<'a, T: ?Sized> WriteGuard<'a, T> {
    pub(crate) fn new(r: TraceMut<'a, T>, ll: &'a ListenerList) -> Self {
        ll.callback_all(CallbackAction::RegisterDirty);
        Self {
            r,
            wl: WakeListeners { ll, mutated: false },
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
        self.wl.mutated = true;
        &mut self.r
    }
}

struct WakeListeners<'a> {
    ll: &'a ListenerList,
    mutated: bool,
}

impl Drop for WakeListeners<'_> {
    fn drop(&mut self) {
        self.ll.callback_all(if self.mutated {
            CallbackAction::Update
        } else {
            CallbackAction::ClearDirty
        });
    }
}
