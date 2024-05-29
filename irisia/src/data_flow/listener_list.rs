use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::Listener;

thread_local! {
    static STACK: RefCell<Vec<Listener>> = Default::default();
}

type ListenerTable = RefCell<HashMap<*const (), Listener>>;

#[derive(Default)]
pub(super) struct ListenerList {
    current: ListenerTable,
    previous: ListenerTable,
}

impl ListenerList {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push_global_stack(l: Listener) {
        STACK.with_borrow_mut(|vec| vec.push(l));
    }

    pub fn pop_global_stack() {
        STACK.with_borrow_mut(|vec| vec.pop());
    }

    pub fn capture_caller(&self) {
        STACK.with_borrow(|vec| {
            if let Some(le) = vec.last() {
                self.watch(le)
            }
        });
    }

    pub fn watch(&self, le: &Listener) {
        self.current.borrow_mut().insert(
            match le {
                Listener::Once(wire) => wire.as_ptr() as _,
                Listener::LongLived(watcher) => Rc::as_ptr(watcher) as _,
            },
            le.clone(),
        );
    }

    pub fn wake_all(&self) {
        let mut previous = self
            .previous
            .try_borrow_mut()
            .expect("cannot assign to a data source when this source is updating");

        debug_assert!(previous.is_empty());

        std::mem::swap(&mut *self.current.borrow_mut(), &mut *previous);

        previous.retain(|_, listener| match listener {
            Listener::Once(weak) => {
                if let Some(wire) = weak.upgrade() {
                    wire.update();
                }
                false
            }

            Listener::LongLived(watcher) => watcher.clone().update(),
        });

        self.current.borrow_mut().extend(previous.drain());
    }
}
