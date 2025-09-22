use std::marker::PhantomData;

pub use proxy_signal::SignalProxied;

use crate::Signal;

pub mod proxy_signal;

pub trait Definition {
    type Value;
    type Storage: 'static;

    fn create(&self) -> (Self::Storage, Self::Value);
    fn update(&self, storage: &mut Self::Storage);
}

// DirectAssign

pub struct DirectAssign<T>(pub T);

impl<T: Clone> Definition for DirectAssign<T> {
    type Value = T;
    type Storage = ();

    fn create(&self) -> (Self::Storage, Self::Value) {
        ((), self.0.clone())
    }

    fn update(&self, _: &mut Self::Storage) {}
}

// UsingDefault

pub struct UsingDefault<T: ?Sized>(PhantomData<Option<Signal<T>>>);

impl<T: ?Sized> UsingDefault<T> {
    pub const GET: Self = UsingDefault(PhantomData);
}

impl<T: ?Sized> Definition for UsingDefault<T> {
    type Value = Option<Signal<T>>;
    type Storage = ();
    fn create(&self) -> (Self::Storage, Self::Value) {
        ((), None)
    }
    fn update(&self, _: &mut Self::Storage) {}
}

// MapValue

pub struct 