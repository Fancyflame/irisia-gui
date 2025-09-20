use crate::{
    hook::Signal,
    model::component::{definition::Definition, property::PropAssign},
};
use std::marker::PhantomData;

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

impl<T> PropAssign<DirectAssign<T>> for T
where
    T: Clone,
{
    fn prop_assign(value: T) -> Self {
        value
    }
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

// Chain

impl<T, U> Definition for (T, U)
where
    T: Definition,
    U: Definition,
{
    type Storage = (T::Storage, U::Storage);
    type Value = (T::Value, U::Value);

    fn create(&self) -> (Self::Storage, Self::Value) {
        let (s1, v1) = self.0.create();
        let (s2, v2) = self.1.create();
        ((s1, s2), (v1, v2))
    }

    fn update(&self, storage: &mut Self::Storage) {
        self.0.update(&mut storage.0);
        self.1.update(&mut storage.1);
    }
}

// Empty

impl Definition for () {
    type Value = ();
    type Storage = ();

    fn create(&self) -> (Self::Storage, Self::Value) {
        ((), ())
    }
    fn update(&self, _: &mut Self::Storage) {}
}
