use std::rc::Rc;

use write_guard::ReactiveWriteGuard;

use super::utils::trace_cell::TraceRef;
use inner::Inner;

mod builder;
mod coerce;
mod inner;
mod write_guard;

pub struct Reactive<T: ?Sized> {
    inner: Rc<Inner<T>>,
}

impl<T: ?Sized> Reactive<T> {
    pub fn read(&self) -> TraceRef<T> {
        self.inner
            .value
            .borrow()
            .expect("cannot get immutable value")
    }

    pub fn write(&self) -> ReactiveWriteGuard<T> {
        ReactiveWriteGuard {
            r: self
                .inner
                .value
                .borrow_mut()
                .expect("cannot get mutable value"),
            inner: &self.inner,
        }
    }
}

impl<T: ?Sized> Clone for Reactive<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
