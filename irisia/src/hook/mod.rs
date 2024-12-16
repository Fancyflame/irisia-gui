use std::{ops::Deref, rc::Rc};

use utils::trace_cell::TraceRef;

pub use {effect::Effect, listener::Listener, signal::Signal, simple::SimpleProvider};

pub mod effect;
pub mod listener;
pub mod provider_group;
pub mod signal;
pub mod simple;
pub mod utils;

pub trait Provider {
    type Data: ?Sized;
    fn read(&self) -> Ref<Self::Data>;
    fn dependent(&self, listener: Listener);
}

pub enum Ref<'a, T: ?Sized> {
    Ref(&'a T),
    TraceRef(TraceRef<'a, T>),
}

impl<T: ?Sized> Deref for Ref<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Ref(r) => r,
            Self::TraceRef(r) => &r,
        }
    }
}

pub struct ProviderObject<T: ?Sized>(Signal<T>);

impl<T: ?Sized> Provider for ProviderObject<T> {
    type Data = T;
    fn dependent(&self, listener: Listener) {
        self.0.dependent(listener);
    }
    fn read(&self) -> Ref<Self::Data> {
        self.0.read()
    }
}

impl<T: ?Sized> Clone for ProviderObject<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub trait ToProviderObject {
    type Data: ?Sized;
    fn to_object(&self) -> ProviderObject<Self::Data>;
}

impl<T> ToProviderObject for ProviderObject<T> {
    type Data = T;
    fn to_object(&self) -> ProviderObject<T> {
        self.clone()
    }
}

impl<T: ToProviderObject> ToProviderObject for &T {
    type Data = T::Data;
    fn to_object(&self) -> ProviderObject<Self::Data> {
        (**self).to_object()
    }
}
