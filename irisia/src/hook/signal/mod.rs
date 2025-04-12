use std::rc::Rc;

use builder::SignalBuilder;

use super::{
    signal_group::SignalGroup,
    utils::{trace_cell::TraceRef, WriteGuard},
    Listener,
};
use inner::Inner;

mod builder;
mod coerce;
mod inner;

pub struct Signal<T: ?Sized> {
    inner: Rc<Inner<T>>,
}

impl<T: 'static> Signal<T> {
    pub fn state(value: T) -> WriteSignal<T> {
        Self::builder(value).writable().build()
    }

    pub fn memo<F, D>(generator: F, deps: D) -> Self
    where
        T: PartialEq<T>,
        F: Fn(D::Data<'_>) -> T + 'static,
        D: SignalGroup + 'static,
    {
        let builder = Self::builder(generator(D::deref_wrapper(&deps.read_many())));
        builder
            .dep(
                move |mut this, data| {
                    let new_value = generator(data);
                    if *this != new_value {
                        *this = new_value;
                    }
                },
                deps,
            )
            .build()
    }

    pub fn memo_ncmp<F, D>(generator: F, deps: D) -> Self
    where
        F: Fn(D::Data<'_>) -> T + 'static,
        D: SignalGroup + 'static,
    {
        let builder = Self::builder(generator(D::deref_wrapper(&deps.read_many())));
        builder
            .dep(
                move |mut this, data| {
                    *this = generator(data);
                },
                deps,
            )
            .build()
    }

    pub fn builder(value: T) -> SignalBuilder<T, (), ()> {
        SignalBuilder {
            value,
            callbacks: (),
            writable: (),
        }
    }
}

impl<T: ?Sized> Signal<T> {
    pub fn read(&self) -> TraceRef<T> {
        self.inner.read()
    }

    pub fn dependent(&self, l: Listener) {
        self.inner.dependent(l);
    }

    pub(crate) fn addr(&self) -> *const () {
        Rc::as_ptr(&self.inner) as _
    }
}

pub struct WriteSignal<T: ?Sized>(Signal<T>);

impl<T: ?Sized> WriteSignal<T> {
    pub fn write(&self) -> WriteGuard<T> {
        WriteGuard::new(
            self.0.inner.value.borrow_mut().unwrap(),
            &self.0.inner.listeners,
        )
    }

    pub fn set(&self, data: T)
    where
        T: Sized,
    {
        *self.write() = data;
    }

    pub fn read(&self) -> TraceRef<T> {
        self.0.read()
    }

    pub fn to_signal(&self) -> Signal<T> {
        self.0.clone()
    }
}

impl<T: ?Sized> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: ?Sized> Clone for WriteSignal<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
