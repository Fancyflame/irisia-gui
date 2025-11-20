use std::rc::{Rc, Weak};

use crate::hook::signal_group::SignalGroup;

use super::utils::{CallbackAction, DirtyCount};

#[derive(Clone)]
pub struct Listener(Weak<dyn ListenerInner>);

/// DON'T FORGET to call `start_listen()`.
#[derive(Clone)]
pub(crate) struct StrongListener(#[allow(dead_code)] Rc<dyn ListenerInner>);

impl StrongListener {
    pub fn start_listen(&self) {
        self.0.start_listen(self);
    }
}

impl Listener {
    /// The callback **must NOT capture hooks** or will cause underlying memory leaks.
    ///
    /// And **DONT FORGET** to call `start_listen()`
    pub(crate) fn new<D, F>(deps: D, callback: F) -> StrongListener
    where
        D: SignalGroup + 'static,
        F: Fn(CallbackAction, &D) -> bool + 'static,
    {
        let inner = Rc::new(Inner {
            dirty_count: DirtyCount::new(),
            callback,
            deps,
        });

        StrongListener(inner)
    }

    pub(crate) fn callback(&self, action: CallbackAction) -> bool {
        let Some(rc) = self.0.upgrade() else {
            return false;
        };
        rc.push_action(action)
    }
}

struct Inner<F, D> {
    dirty_count: DirtyCount,
    callback: F,
    deps: D,
}

pub(super) trait ListenerInner {
    fn push_action(&self, action: CallbackAction) -> bool;
    fn start_listen(&self, self_as_listener: &StrongListener);
}

impl<F, D> ListenerInner for Inner<F, D>
where
    F: Fn(CallbackAction, &D) -> bool,
    D: SignalGroup,
{
    fn push_action(&self, action: CallbackAction) -> bool {
        match self.dirty_count.push(action) {
            Some(spread_action) => (self.callback)(spread_action, &self.deps),
            None => true,
        }
    }

    fn start_listen(&self, self_as_listener: &StrongListener) {
        self.deps
            .dependent_many(Listener(Rc::downgrade(&self_as_listener.0)));
    }
}
