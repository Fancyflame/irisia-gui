use std::rc::Rc;

use crate::hook::check_casted_rc;

use super::{inner::Inner, Reactive};

type CoerceFn<T, U> = fn(Rc<Inner<T>>) -> Rc<Inner<U>>;

impl<T: ?Sized> Reactive<T> {
    #[doc(hidden)]
    pub fn __irisia_coerce_unsized<U>(&self, map: CoerceFn<T, U>) -> Reactive<U>
    where
        U: ?Sized,
    {
        let new = map(self.inner.clone());
        check_casted_rc(&self.inner, &new);

        Reactive { inner: new }
    }
}
