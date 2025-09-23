use std::marker::PhantomData;

pub use proxy_signal::SignalProxied;

use crate::{Property, Signal};

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

type MapValueFn<D> = fn(
    rest: <D as Definition>::Value,
    overwrite: <D as Definition>::Value,
) -> <D as Definition>::Value;

pub struct MapValue<D: Definition> {
    definition: D,
    rest_value: D::Value,
    func: MapValueFn<D>,
}

pub fn map_definition<D>(definition: D, v: D::Value, func: MapValueFn<D>) -> MapValue<D>
where
    D: Definition,
    D::Value: Property,
{
    MapValue {
        definition,
        rest_value: v,
        func,
    }
}

impl<D> Definition for MapValue<D>
where
    D: Definition,
    D::Value: Property,
{
    type Value = D::Value;
    type Storage = D::Storage;

    fn create(&self) -> (Self::Storage, Self::Value) {
        let (storage, value) = self.definition.create();
        (
            storage,
            (self.func)(self.rest_value.property_clone(), value),
        )
    }

    fn update(&self, storage: &mut Self::Storage) {
        self.definition.update(storage);
    }
}

// Prevent Type

impl Definition for () {
    type Storage = ();
    type Value = ();

    fn create(&self) -> (Self::Storage, Self::Value) {
        ((), ())
    }
    fn update(&self, _: &mut Self::Storage) {}
}
