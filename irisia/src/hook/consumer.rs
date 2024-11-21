use std::{cell::RefCell, rc::Rc};

use super::{listener::CallbackAction, provider_group::ProviderGroup, Listener};

pub struct Consumer<T: ?Sized> {
    inner: Rc<Inner<T>>,
}

struct Inner<T: ?Sized> {
    value: RefCell<T>,
}

impl<T: ?Sized + 'static> Consumer<T> {
    pub fn new(value: T) -> Self
    where
        T: Sized,
    {
        Self {
            inner: Rc::new(Inner {
                value: RefCell::new(value),
            }),
        }
    }

    pub fn dep<F, D>(&self, callback: F, deps: D) -> &Self
    where
        F: Fn(&mut T) + 'static,
        D: ProviderGroup,
    {
        let listener = Listener::new(Rc::downgrade(&self.inner), move |this, action| {
            if let CallbackAction::Update = action {
                callback(&mut this.value.borrow_mut());
            }
        });
        deps.dependent_many(listener);
        self
    }

    pub fn borrow(&self) -> std::cell::Ref<T> {
        self.inner.value.borrow()
    }

    pub fn borrow_mut(&mut self) -> std::cell::RefMut<T> {
        self.inner.value.borrow_mut()
    }
}

impl<T: ?Sized> Clone for Consumer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
