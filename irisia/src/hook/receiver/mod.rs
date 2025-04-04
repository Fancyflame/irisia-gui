use std::rc::Rc;

use builder::ReceiverBuilder;

use super::utils::trace_cell::{TraceMut, TraceRef};
use inner::Inner;

mod builder;
mod inner;

pub struct Receiver<T: ?Sized> {
    inner: Rc<Inner<T>>,
}

impl<T: 'static> Receiver<T> {
    pub fn builder(value: T) -> ReceiverBuilder<T, ()> {
        ReceiverBuilder {
            value,
            callbacks: (),
        }
    }
}

impl<T: ?Sized> Receiver<T> {
    pub fn read(&self) -> TraceRef<T> {
        self.inner
            .value
            .borrow()
            .expect("cannot get immutable value")
    }

    pub fn write(&self) -> TraceMut<T> {
        self.inner
            .value
            .borrow_mut()
            .expect("cannot get mutable value")
    }
}

impl<T: ?Sized> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
