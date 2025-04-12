use super::{signal_group::SignalGroup, utils::trace_cell::TraceRef};
use builder::ReactiveRef;
pub use builder::RealRef;
use inner::Inner;
use std::rc::Rc;
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

    pub fn into_inner(self) -> Option<T>
    where
        T: Sized,
    {
        Rc::into_inner(self.inner).map(|inner| inner.value.into_inner())
    }
}

impl<T> Clone for Reactive<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
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
