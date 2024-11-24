use std::rc::Rc;

use builder::ConsumerBuilder;

use super::provider_group::ProviderGroup;

mod builder;
mod callback_chain;

mod inner {
    pub struct Inner<T, C: ?Sized = dyn Never> {
        pub(super) value: std::cell::RefCell<T>,
        pub(super) callbacks: C,
    }

    pub trait Never {}
    impl<T> Never for T {}
}
use inner::Inner;

pub struct Consumer<T> {
    inner: Rc<Inner<T>>,
}

impl<T: 'static> Consumer<T> {
    pub fn new<F, D>(value: T, callback: F, deps: D) -> Self
    where
        F: Fn(&mut T, D::Data<'_>) + 'static,
        D: ProviderGroup + 'static,
    {
        Self::builder(value).dep(callback, deps).build()
    }

    pub fn builder(value: T) -> ConsumerBuilder<T, ()> {
        ConsumerBuilder {
            value,
            callbacks: (),
        }
    }

    pub fn borrow(&self) -> std::cell::Ref<T> {
        self.inner.value.borrow()
    }

    pub fn borrow_mut(&mut self) -> std::cell::RefMut<T> {
        self.inner.value.borrow_mut()
    }
}

impl<T> Clone for Consumer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
