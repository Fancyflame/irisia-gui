use std::any::Any;

use crate::hook::{
    utils::{trace_cell::TraceRef, CallbackAction, DirtyCount, ListenerList, TraceCell},
    Listener,
};

pub struct Inner<T: ?Sized> {
    pub(super) listeners: ListenerList,
    pub(super) global_dirty_count: DirtyCount,
    pub(super) callback_chain_storage: Box<dyn Any>,
    pub(super) value: TraceCell<T>,
}

impl<T> Inner<T>
where
    T: ?Sized,
{
    pub fn read(&self) -> TraceRef<T> {
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
