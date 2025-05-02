use std::rc::{Rc, Weak};

use super::utils::{CallbackAction, DirtyCount};

type Core = Inner<dyn Fn(CallbackAction) -> bool>;

pub struct Listener(Weak<Core>);

pub(crate) struct StrongListener(#[allow(dead_code)] Rc<Core>);

impl StrongListener {
    pub fn downgrade(&self) -> Listener {
        Listener(Rc::downgrade(&self.0))
    }
}

impl Listener {
    /// The callback ***must NOT capture hooks*** or will cause underlying memory leaks
    pub(crate) fn new<Maker, F>(make_callback: Maker) -> StrongListener
    where
        Maker: FnOnce(Listener) -> F,
        F: Fn(CallbackAction) -> bool + 'static,
    {
        let inner = Rc::new_cyclic(|weak_inner| Inner {
            dirty_count: DirtyCount::new(),
            callback: make_callback(Listener(weak_inner.clone() as _)),
        });

        StrongListener(inner)
    }

    pub(crate) fn callback(&self, action: CallbackAction) -> bool {
        let Some(rc) = self.0.upgrade() else {
            return false;
        };

        let Some(spread_action) = rc.dirty_count.push(action) else {
            return true;
        };

        (rc.callback)(spread_action)
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
