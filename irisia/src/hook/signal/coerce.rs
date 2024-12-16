use std::rc::Rc;

use super::{inner::Inner, Signal};

#[macro_export]
macro_rules! coerce_signal {
    ($signal:expr) => {
        $crate::hook::signal::Signal::__coerce_unsized($signal, |inner| inner as _)
    };
}

impl<T: ?Sized> Signal<T> {
    #[doc(hidden)]
    pub fn __coerce_unsized<U>(self, map: fn(Rc<Inner<T>>) -> Rc<Inner<U>>) -> Signal<U>
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
