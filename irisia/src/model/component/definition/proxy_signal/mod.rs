use crate::{Signal, hook::signal::WriteSignal, model::component::definition::Definition};

pub mod helper;

pub struct SignalProxied<T, U = T>
where
    U: ?Sized,
{
    value: T,
    eq_fn: fn(&T, &T) -> bool,
    map_value: fn(Signal<T>) -> Signal<U>,
}

impl<T, U> SignalProxied<T, U>
where
    U: ?Sized,
{
    pub fn coerce_unsize<U2>(self, f: fn(Signal<T>) -> Signal<U2>) -> SignalProxied<T, U2>
    where
        U2: ?Sized,
    {
        SignalProxied {
            value: self.value,
            eq_fn: self.eq_fn,
            map_value: f,
        }
    }

    pub fn coerce_unsize_helped<U2>(
        self,
        _: impl Fn(SignalProxied<(), U2>),
    ) -> impl FnOnce(fn(Signal<T>) -> Signal<U2>) -> SignalProxied<T, U2>
    where
        U2: ?Sized,
    {
        |f| self.coerce_unsize(f)
    }
}

pub struct SignalHoster<T> {
    signal: WriteSignal<T>,
}

impl<T, U> Definition for SignalProxied<T, U>
where
    T: Clone + 'static,
    U: ?Sized,
{
    type Value = Signal<U>;
    type Storage = SignalHoster<T>;

    fn create(&self) -> (Self::Storage, Self::Value) {
        let signal = Signal::state(self.value.clone());
        (
            SignalHoster {
                signal: signal.clone(),
            },
            (self.map_value)(signal.to_read()),
        )
    }

    fn update(&self, storage: &mut Self::Storage) {
        let mut w = storage.signal.write();
        if !(self.eq_fn)(&*w, &self.value) {
            *w = self.value.clone();
        }
    }
}
