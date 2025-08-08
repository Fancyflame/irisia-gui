use std::rc::Rc;

use super::{Signal, WriteSignal, inner::Inner};
use crate::hook::check_casted_rc;

mod fast_coerce;

type CoerceFn<T, U> = fn(Rc<Inner<T>>) -> Rc<Inner<U>>;

impl<T: ?Sized> Signal<T> {
    #[doc(hidden)]
    pub fn __irisia_coerce_unsized<U>(&self, map: CoerceFn<T, U>) -> Signal<U>
    where
        U: ?Sized,
    {
        let new = map(self.inner.clone());
        check_casted_rc(&self.inner, &new);

        Signal { inner: new }
    }
}

impl<T: ?Sized> WriteSignal<T> {
    #[doc(hidden)]
    pub fn __irisia_coerce_unsized<U>(&self, map: CoerceFn<T, U>) -> WriteSignal<U>
    where
        U: ?Sized,
    {
        WriteSignal(self.0.__irisia_coerce_unsized(map))
    }
}
