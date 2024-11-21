use std::{ops::Deref, rc::Rc};

pub use listener::Listener;

pub mod constant;
pub mod consumer;
pub mod listener;
pub mod memo;
pub mod provider_group;
pub mod state;
pub mod trace_cell;
pub mod utils;

#[cfg(test)]
mod test;

pub trait Provider {
    type Data;
    fn read(&self) -> Ref<Self::Data>;
    fn dependent(&self, listener: Listener);
}

pub trait ToProviderObject {
    type Data;
    fn to_object(&self) -> ProviderObject<Self::Data>;
}

pub enum Ref<'a, T: ?Sized> {
    Ref(&'a T),
    TraceRef(trace_cell::TraceRef<'a, T>),
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

pub struct ProviderObject<T>(Rc<dyn Provider<Data = T>>);

impl<T> Provider for ProviderObject<T> {
    type Data = T;
    fn dependent(&self, listener: Listener) {
        self.0.dependent(listener);
    }
    fn read(&self) -> Ref<Self::Data> {
        self.0.read()
    }
}

impl<T> ToProviderObject for ProviderObject<T> {
    type Data = T;
    fn to_object(&self) -> ProviderObject<T> {
        self.clone()
    }
}

impl<T> Clone for ProviderObject<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
