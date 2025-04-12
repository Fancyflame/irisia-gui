use std::ops::{Deref, DerefMut};

use super::definition::SignalProxied;

pub struct CheckEq<T>(Fallback<T>);
pub struct Fallback<T>(Option<T>);
const FALLBACK_TO: bool = false;

pub fn check_eq<T>(value: T) -> CheckEq<T> {
    CheckEq(Fallback(Some(value)))
}

impl<T: PartialEq<T>> CheckEq<T> {
    pub fn get(self) -> SignalProxied<T> {
        SignalProxied {
            value: self.0 .0.unwrap(),
            eq_fn: T::eq,
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
            eq_fn: |_, _| FALLBACK_TO,
        }
    }
}

#[test]
fn test() {
    let _ = check_eq(100).get();

    struct NotImplementEq;
    let _ = check_eq(NotImplementEq).get();
}
