use smallvec::SmallVec;
use std::cell::RefCell;

use crate::hook::{utils::CallbackAction, Listener};

/// Use it when implementing a provider
#[derive(Default)]
pub struct ListenerList {
    listeners: RefCell<Vec<Listener>>,
    delay_add: RefCell<SmallVec<[Listener; 1]>>,
}

impl ListenerList {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_listener(&self, listener: Listener) {
        match self.listeners.try_borrow_mut() { Ok(mut refmut) => {
            refmut.push(listener);
        } _ => {
            self.delay_add.borrow_mut().push(listener);
        }}
    }

    fn for_each_listeners<F>(&self, f: F)
    where
        F: FnMut(&Listener) -> bool,
    {
        let mut listeners = self.listeners.try_borrow_mut().expect(
            "cannot operate listeners because it is already in use (borrowed as mutable). \
            please check if you are attempt to dirt or wake listener when this listener is \
            being dirting or waking.",
        );

        listeners.retain(f);
        listeners.extend(self.delay_add.borrow_mut().drain(..));
    }

    pub fn callback_all(&self, action: CallbackAction) {
        self.for_each_listeners(|listener| listener.callback(action));
    }
}
