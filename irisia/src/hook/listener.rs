use std::rc::{Rc, Weak};

use super::utils::{CallbackAction, DirtyCount};

#[derive(Clone)]
pub struct Listener(Weak<dyn ListenerInnerTrait>);

impl Listener {
    /// The callback **must NOT capture hooks** or will cause underlying memory leaks.
    ///
    /// And **DONT FORGET** to call `start_listen()`
    pub(super) fn new<F>(callback: F) -> Rc<ListenerInner<F>>
    where
        F: ListenerCallback,
    {
        Rc::new(ListenerInner {
            dirty_count: DirtyCount::new(),
            callback,
        })
    }

    pub(crate) fn callback(&self, action: CallbackAction) -> bool {
        let Some(rc) = self.0.upgrade() else {
            return false;
        };
        rc.push_action(action)
    }
}

pub(super) struct ListenerInner<F: ?Sized> {
    dirty_count: DirtyCount,
    pub callback: F,
}

impl<F> ListenerInner<F>
where
    F: ListenerCallback,
{
    pub fn to_listener(self: &Rc<Self>) -> Listener {
        Listener(Rc::downgrade(self) as _)
    }
}

trait ListenerInnerTrait {
    fn push_action(&self, action: CallbackAction) -> bool;
}

impl<F> ListenerInnerTrait for ListenerInner<F>
where
    F: ListenerCallback + ?Sized,
{
    fn push_action(&self, action: CallbackAction) -> bool {
        match self.dirty_count.push(action) {
            Some(spread_action) => self.callback.call(spread_action),
            None => true,
        }
    }
}

pub(super) trait ListenerCallback: 'static {
    fn call(&self, action: CallbackAction) -> bool;
}
