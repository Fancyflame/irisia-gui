use smallvec::SmallVec;

use crate::hook::{
    Listener,
    listener::StrongListener,
    utils::{CallbackAction, DirtyCount, ListenerList, TraceCell, trace_cell::TraceRef},
};

pub struct StrongListenerList(pub(super) SmallVec<[StrongListener; 1]>);

pub struct Inner<T: ?Sized> {
    pub(super) listeners: ListenerList,
    pub(super) global_dirty_count: DirtyCount,
    pub(super) _store_list: StrongListenerList,
    pub(super) value: TraceCell<T>,
}

impl<T> Inner<T>
where
    T: ?Sized,
{
    pub fn read(&self) -> TraceRef<'_, T> {
        self.value.borrow().unwrap()
    }

    pub fn dependent(&self, listener: Listener) {
        self.listeners.add_listener(listener);
    }
}

impl<T: ?Sized> Inner<T> {
    pub(super) fn push_action(&self, action: CallbackAction) {
        if let Some(action) = self.global_dirty_count.push(action) {
            self.listeners.callback_all(action);
        };
    }
}
