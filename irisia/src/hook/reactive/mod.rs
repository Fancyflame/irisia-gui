use std::rc::Rc;

use builder::ReceiverBuilder;

use super::utils::trace_cell::{TraceMut, TraceRef};
use inner::Inner;

mod builder;
mod coerce;
mod inner;

pub struct Reactive<T: ?Sized> {
    inner: Rc<Inner<T>>,
}

impl<T: 'static> Reactive<T> {
    pub fn builder(value: T) -> ReceiverBuilder<T, ()> {
        ReceiverBuilder {
            value,
            callbacks: (),
        }
    }
}

impl<T: ?Sized> Reactive<T> {
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

impl<T: ?Sized> Clone for Reactive<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
