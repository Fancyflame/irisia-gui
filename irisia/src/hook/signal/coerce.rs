use std::rc::Rc;

use crate::hook::ProviderObject;

use super::{inner::Inner, Signal};

#[macro_export]
macro_rules! coerce_signal {
    ($signal:expr) => {
        (($signal).__irisia_coerce_unsized(|inner| inner as _))
    };
}

type CoerceFn<T, U> = fn(Rc<Inner<T>>) -> Rc<Inner<U>>;

impl<T: ?Sized> Signal<T> {
    #[doc(hidden)]
    pub fn __irisia_coerce_unsized<U>(&self, map: CoerceFn<T, U>) -> Signal<U>
    where
        U: ?Sized,
    {
        let new = map(self.inner.clone());
        assert_eq!(
            Rc::as_ptr(&self.inner) as *const _ as *const (),
            Rc::as_ptr(&new) as *const _ as *const (),
            "the returned Rc pointer address is not equal to the input Rc"
        );

        Signal { inner: new }
    }
}

impl<T: ?Sized> ProviderObject<T> {
    #[doc(hidden)]
    pub fn __irisia_coerce_unsized<U>(&self, map: CoerceFn<T, U>) -> ProviderObject<U>
    where
        U: ?Sized,
    {
        ProviderObject(self.0.__irisia_coerce_unsized(map))
    }
}
