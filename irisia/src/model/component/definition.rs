use crate::hook::{signal::WriteSignal, Signal};

pub trait Definition {
    type Value;
    type Storage: 'static;

    fn create(&self) -> (Self::Storage, Self::Value);
    fn update(&self, storage: &mut Self::Storage);
}

pub struct SignalProxied<T> {
    pub value: T,
    pub eq_fn: fn(&T, &T) -> bool,
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
            signal.to_signal(),
        )
    }

    fn update(&self, storage: &mut Self::Storage) {
        let mut w = storage.signal.write();
        if !(self.eq_fn)(&*w, &self.value) {
            *w = self.value.clone();
        }
    }
}

pub struct DirectAssign<T>(pub T);

impl<T: Clone> Definition for DirectAssign<T> {
    type Value = T;
    type Storage = ();

    fn create(&self) -> (Self::Storage, Self::Value) {
        ((), self.0.clone())
    }

    fn update(&self, _: &mut Self::Storage) {}
}

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

impl Definition for () {
    type Value = ();
    type Storage = ();

    fn create(&self) -> (Self::Storage, Self::Value) {
        ((), ())
    }
    fn update(&self, _: &mut Self::Storage) {}
}
