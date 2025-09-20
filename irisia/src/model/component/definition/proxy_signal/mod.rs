use crate::{
    Signal,
    hook::signal::WriteSignal,
    model::component::{definition::Definition, property::PropAssign},
};

pub mod helper;

pub struct SignalProxied<T> {
    pub(super) value: T,
    pub(super) eq_fn: fn(&T, &T) -> bool,
}

pub struct SignalHoster<T> {
    signal: WriteSignal<T>,
}

impl<T: Clone + 'static> Definition for SignalProxied<T> {
    type Value = Signal<T>;
    type Storage = SignalHoster<T>;

    fn create(&self) -> (Self::Storage, Self::Value) {
        let signal = Signal::state(self.value.clone());
        (
            SignalHoster {
                signal: signal.clone(),
            },
            signal.to_read(),
        )
    }

    fn update(&self, storage: &mut Self::Storage) {
        let mut w = storage.signal.write();
        if !(self.eq_fn)(&*w, &self.value) {
            *w = self.value.clone();
        }
    }
}

impl<T> PropAssign<SignalProxied<T>> for Signal<T>
where
    T: Clone + 'static,
{
    fn prop_assign(value: Signal<T>) -> Self {
        value
    }
}

impl<T> PropAssign<SignalProxied<T>> for Option<Signal<T>>
where
    T: Clone + 'static,
{
    fn prop_assign(value: Signal<T>) -> Self {
        Some(value)
    }
}
