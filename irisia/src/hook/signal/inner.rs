use std::rc::Weak;

use crate::hook::{
    utils::{ListenerList, TraceCell},
    Listener, Provider, Ref,
};

pub struct Inner<T, C: ?Sized = dyn Noop> {
    pub(super) value: TraceCell<T>,
    pub(super) listeners: ListenerList,
    pub(super) as_provider: Weak<dyn Provider<Data = T>>,
    pub(super) callbacks: C,
}

impl<T, C> Provider for Inner<T, C>
where
    C: ?Sized,
{
    type Data = T;
    fn read(&self) -> Ref<Self::Data> {
        Ref::TraceRef(self.value.borrow().unwrap())
    }
    fn dependent(&self, listener: Listener) {
        self.listeners.add_listener(listener);
    }
}

pub trait Noop {}
impl<T> Noop for T {}
