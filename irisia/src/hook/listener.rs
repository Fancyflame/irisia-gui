use std::rc::{Rc, Weak};

use super::utils::DirtyCount;

pub struct Listener(Rc<dyn Callback>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CallbackAction {
    Update,
    RegisterDirty,
    ClearDirty,
}

impl Listener {
    /// The callback ***must NOT capture hooks*** or will cause underlying memory leaks
    pub(crate) fn new<T, F>(src: Weak<T>, callback: F) -> Self
    where
        T: ?Sized + 'static,
        F: Fn(&T, CallbackAction) + 'static,
    {
        let inner = Inner {
            src,
            callback,
            dirty_count: DirtyCount::new(),
        };
        Listener(Rc::new(inner))
    }

    pub(crate) fn callback(&self, action: CallbackAction) -> bool {
        self.0.callback(action)
    }
}

struct Inner<T: ?Sized, F> {
    src: Weak<T>,
    callback: F,
    dirty_count: DirtyCount,
}

trait Callback {
    fn callback(&self, action: CallbackAction) -> bool;
}

impl<T, F> Callback for Inner<T, F>
where
    T: ?Sized + 'static,
    F: Fn(&T, CallbackAction) + 'static,
{
    fn callback(&self, action: CallbackAction) -> bool {
        let rc = match self.src.upgrade() {
            Some(rc) => rc,
            None => return false,
        };

        if let Some(spread_action) = self.dirty_count.push(action) {
            (self.callback)(&rc, spread_action);
        };

        true
    }
}

impl Clone for Listener {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl CallbackAction {
    pub fn is_update(&self) -> bool {
        matches!(self, Self::Update)
    }
}
