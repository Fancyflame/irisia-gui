use std::{cell::RefCell, rc::Rc};

use super::{listener::CallbackAction, read_many::ProviderGroup, Listener};

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

    pub fn with_dep<D>(&self, callback: fn(&mut T), deps: D) -> &Self
    where
        D: ProviderGroup,
    {
        let listener = Listener::new(Rc::downgrade(&self.inner), move |this, action| {
            if let CallbackAction::Update = action {
                this.callback(callback, action);
            }
        });
        deps.dependent_many(listener);
        self
    }

    pub fn with_dep_action<D>(&self, callback: fn(&mut T, CallbackAction), deps: D) -> &Self
    where
        D: ProviderGroup,
    {
        let listener = Listener::new(Rc::downgrade(&self.inner), move |this, action| {
            this.callback(|value| callback(value, action), action);
        });
        deps.dependent_many(listener);
        self
    }
}

impl<T: ?Sized> Inner<T> {
    fn callback<F>(&self, f: F, _action: CallbackAction)
    where
        F: Fn(&mut T),
    {
        f(&mut self.value.borrow_mut());
    }
}

impl<T: ?Sized> Clone for Consumer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
