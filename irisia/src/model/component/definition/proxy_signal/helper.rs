use std::ops::{Deref, DerefMut};

use crate::Signal;

use super::SignalProxied;

pub struct CheckEq<T>(Fallback<T>);
pub struct Fallback<T>(Option<T>);
const EQ_FALLBACK_TO: bool = false;

pub fn check_eq<T>(value: T) -> CheckEq<T> {
    CheckEq(Fallback(Some(value)))
}

impl<T: PartialEq<T>> CheckEq<T> {
    pub fn get(self) -> SignalProxied<T> {
        SignalProxied {
            value: self.0.0.unwrap(),
            eq_fn: T::eq,
            map_value: |x| x,
        }
    }
}

impl<T> Deref for CheckEq<T> {
    type Target = Fallback<T>;
    fn deref(&self) -> &Self::Target {
        panic!("don't call deref on CheckEq, call deref_mut instead");
    }
}

impl<T> DerefMut for CheckEq<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Fallback<T> {
    pub fn get(&mut self) -> SignalProxied<T> {
        SignalProxied {
            value: self.0.take().unwrap(),
            eq_fn: |_, _| EQ_FALLBACK_TO,
            map_value: |x| x,
        }
    }
}

// type MapSignalFn<T, U> = fn(Signal<T>) -> Signal<U>;
// pub fn coerce_proxy_signal_helper<T, U>(
//     _: impl Fn(SignalProxied<(), U>),
// ) -> fn(MapSignalFn<T, U>) -> MapSignalFn<T, U>
// where
//     U: ?Sized,
// {
//     |x| x
// }

#[test]
fn test() {
    let _ = check_eq(100).get();

    struct NotImplementEq;
    let _ = check_eq(NotImplementEq).get();
}
