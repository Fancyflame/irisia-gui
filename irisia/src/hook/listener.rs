use std::rc::Rc;

use super::utils::{CallbackAction, DirtyCount};

pub struct Listener(Rc<Inner<dyn Fn(CallbackAction) -> bool>>);

impl Listener {
    /// The callback ***must NOT capture hooks*** or will cause underlying memory leaks
    pub(crate) fn new<F>(callback: F) -> Self
    where
        F: Fn(CallbackAction) -> bool + 'static,
    {
        let inner = Inner {
            dirty_count: DirtyCount::new(),
            callback,
        };

        Listener(Rc::new(inner) as _)
    }

    pub(crate) fn callback(&self, action: CallbackAction) -> bool {
        if let Some(spread_action) = self.0.dirty_count.push(action) {
            (self.0.callback)(spread_action)
        } else {
            true
        }
    }
}

struct Inner<F: ?Sized> {
    dirty_count: DirtyCount,
    callback: F,
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
