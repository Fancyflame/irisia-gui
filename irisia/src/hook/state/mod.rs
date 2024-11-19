use std::{ops::Deref, rc::Rc};

use super::{trace_cell::TraceCell, utils::ListenerList, Provider, Ref};

pub use write_guard::WriteGuard;

mod write_guard;

pub struct State<T> {
    inner: Rc<Inner<T>>,
}

struct Inner<T> {
    listener_list: ListenerList,
    value: TraceCell<T>,
}

impl<T> State<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(Inner {
                listener_list: ListenerList::new(),
                value: TraceCell::new(value),
            }),
        }
    }

    pub fn write(&self) -> WriteGuard<T> {
        WriteGuard::new(
            self.inner.value.borrow_mut().unwrap(),
            &self.inner.listener_list,
        )
    }
}

impl<T> Provider for Inner<T> {
    type Data = T;

    fn read(&self) -> Ref<Self::Data> {
        Ref::CellRef(self.value.borrow().unwrap())
    }

    fn dependent(&self, listener: super::Listener) {
        self.listener_list.add_listener(listener)
    }
}

impl<T: 'static> Deref for State<T> {
    type Target = dyn Provider<Data = T>;
    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}
