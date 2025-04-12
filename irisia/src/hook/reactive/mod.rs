use super::{signal_group::SignalGroup, utils::trace_cell::TraceRef};
pub use builder::ReactiveRef;
pub(crate) use builder::RealRef;
use inner::Inner;
use std::rc::{Rc, Weak};
use write_guard::ReactiveWriteGuard;

mod builder;
mod inner;
mod write_guard;

pub struct Reactive<T> {
    inner: Rc<Inner<T>>,
}

impl<T> Reactive<T> {
    pub fn read(&self) -> TraceRef<T> {
        self.inner
            .value
            .borrow()
            .expect("cannot get immutable value")
    }

    pub fn write(&self) -> ReactiveWriteGuard<T> {
        ReactiveWriteGuard::new(
            self.inner
                .value
                .borrow_mut()
                .expect("cannot get mutable value"),
            &self.inner,
        )
    }

    pub fn push<F>(&self, f: F)
    where
        F: FnOnce(ReactiveRef<T>) + 'static,
    {
        match self.inner.value.try_borrow_mut() {
            Some(v) => {
                f(ReactiveRef::Real(RealRef::new(&self.inner.value, v)));
                self.inner.recall_delayed_callback();
            }
            None => self
                .inner
                .delay_callbacks
                .borrow_mut()
                .push_back(Box::new(move |_, r| f(r))),
        }
    }

    pub fn into_inner(self) -> Option<T>
    where
        T: Sized,
    {
        Rc::into_inner(self.inner).map(|inner| inner.value.into_inner())
    }

    pub fn downgrade(&self) -> WeakReactive<T> {
        WeakReactive(Rc::downgrade(&self.inner))
    }
}

impl<T> Clone for Reactive<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

pub struct WeakReactive<T>(Weak<Inner<T>>);

impl<T> WeakReactive<T> {
    pub fn upgrade(&self) -> Option<Reactive<T>> {
        self.0.upgrade().map(|inner| Reactive { inner })
    }
}

impl<T> Clone for WeakReactive<T> {
    fn clone(&self) -> Self {
        WeakReactive(self.0.clone())
    }
}

pub trait CallbackFnAlias<T, D>: FnMut(ReactiveRef<'_, T>, D::Data<'_>) + 'static
where
    D: SignalGroup,
{
}

impl<F, T, D> CallbackFnAlias<T, D> for F
where
    F: FnMut(ReactiveRef<'_, T>, D::Data<'_>) + 'static,
    D: SignalGroup,
{
}
