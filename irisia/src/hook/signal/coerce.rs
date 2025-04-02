use std::rc::Rc;

use super::{inner::Inner, Signal, WriteSignal};

#[macro_export]
macro_rules! coerce_signal {
    ($signal:expr) => {
        (($signal).__irisia_coerce_unsized(|inner| inner as _))
    };
}

type CoerceFn<T, U> = fn(Rc<Inner<T>>) -> Rc<Inner<U>>;

impl<T: ?Sized> Signal<T> {
    #[doc(hidden)]
    pub fn __irisia_coerce_unsized<U>(self, map: CoerceFn<T, U>) -> Signal<U>
    where
        U: ?Sized,
    {
        let prev_ptr = Rc::as_ptr(&self.inner);
        let new = map(self.inner);
        assert_eq!(
            prev_ptr as *const _ as *const (),
            Rc::as_ptr(&new) as *const _ as *const (),
            "the returned Rc pointer address is not equal to the input Rc"
        );

        Signal { inner: new }
    }
}

impl<T: ?Sized> WriteSignal<T> {
    #[doc(hidden)]
    pub fn __irisia_coerce_unsized<U>(self, map: CoerceFn<T, U>) -> WriteSignal<U>
    where
        U: ?Sized,
    {
        WriteSignal(self.0.__irisia_coerce_unsized(map))
    }
}
