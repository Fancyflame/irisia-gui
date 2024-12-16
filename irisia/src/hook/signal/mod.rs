use std::rc::Rc;

use builder::SignalBuilder;

use super::{
    provider_group::ProviderGroup, utils::WriteGuard, Provider, ProviderObject, Ref,
    ToProviderObject,
};
use inner::Inner;

mod builder;
mod coerce;
mod inner;

pub struct Signal<T: ?Sized> {
    inner: Rc<Inner<T>>,
}

impl<T: 'static> Signal<T> {
    pub fn state(value: T) -> Self {
        Self::builder(value).build()
    }

    pub fn memo<F, D>(generator: F, deps: D) -> Self
    where
        T: PartialEq<T>,
        F: Fn(D::Data<'_>) -> T + 'static,
        D: ProviderGroup + 'static,
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

    pub fn builder(value: T) -> SignalBuilder<T, ()> {
        SignalBuilder {
            value,
            callbacks: (),
        }
    }
}

impl<T: ?Sized> Signal<T> {
    pub fn write(&self) -> WriteGuard<T> {
        WriteGuard::new(
            self.inner.value.borrow_mut().unwrap(),
            &self.inner.listeners,
        )
    }
}

impl<T: ?Sized> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: ?Sized> Provider for Signal<T> {
    type Data = T;
    fn read(&self) -> Ref<Self::Data> {
        self.inner.read()
    }
    fn dependent(&self, listener: super::Listener) {
        self.inner.dependent(listener);
    }
}

impl<T: ?Sized> ToProviderObject for Signal<T> {
    type Data = T;
    fn to_object(&self) -> super::ProviderObject<Self::Data> {
        ProviderObject(self.clone())
    }
}
