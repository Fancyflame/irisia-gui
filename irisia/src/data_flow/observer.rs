use std::rc::Rc;

use super::{listener_list::ListenerList, Listener};

pub struct Observer {
    listeners: ListenerList,
}

impl Observer {
    pub fn new() -> Rc<Self> {
        Self {
            listeners: ListenerList::new(),
        }
    }

    pub fn invoke<F, R>(self: &Rc<Self>, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        ListenerList::push_global_stack(Listener::Once())
    }
}
