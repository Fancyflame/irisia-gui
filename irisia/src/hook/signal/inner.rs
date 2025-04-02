use std::any::Any;

use crate::hook::{
    listener::CallbackAction,
    utils::{DirtyCount, ListenerList, TraceCell},
    Listener, Provider, Ref,
};

pub struct Inner<T: ?Sized> {
    pub(super) listeners: ListenerList,
    pub(super) global_dirty_count: DirtyCount,
    pub(super) callback_chain_storage: Box<dyn Any>,
    pub(super) value: TraceCell<T>,
}

impl<T> Provider for Inner<T>
where
    T: ?Sized,
{
    type Data = T;
    fn read(&self) -> Ref<Self::Data> {
        Ref::TraceRef(self.value.borrow().unwrap())
    }
    fn dependent(&self, listener: Listener) {
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
