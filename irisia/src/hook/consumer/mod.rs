use std::{cell::RefCell, rc::Rc};

use super::{Listener, Provider};

pub mod listener;

pub struct Consumer<T: ?Sized> {
    inner: Rc<ResInner<T>>,
}

struct ResInner<T: ?Sized> {
    value: RefCell<T>,
}

impl<T: ?Sized> Consumer<T> {
    pub fn builder(value: T) -> ConsumerBuilder<T>
    where
        T: Sized,
    {
        ConsumerBuilder {
            consumer: Self {
                inner: Rc::new(ResInner {
                    value: RefCell::new(value),
                }),
            },
        }
    }
}

impl<T: ?Sized> Clone for Consumer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

pub struct ConsumerBuilder<T: ?Sized> {
    consumer: Consumer<T>,
}

impl<T: ?Sized + 'static> ConsumerBuilder<T> {
    pub fn dependent_many<'a, P>(
        self,
        providers: P,
        callback: fn(&mut T),
        dirty_setter: Option<fn(&mut T)>,
    ) -> Self
    where
        P: IntoIterator<Item = &'a dyn Provider>,
    {
        let src = Rc::downgrade(&self.consumer.inner);
        let dirty_setter = dirty_setter.unwrap_or(|_| {});

        for provider in providers {
            provider.dependent(Listener::new(src.clone(), callback, dirty_setter));
        }

        self
    }

    pub fn dependent<P>(
        self,
        provider: P,
        callback: fn(&mut T),
        dirty_setter: Option<fn(&mut T)>,
    ) -> Self
    where
        P: Provider,
    {
        provider.dependent(Listener::new(
            Rc::downgrade(&self.consumer.inner),
            callback,
            dirty_setter.unwrap_or(|_| {}),
        ));
        self
    }

    pub fn build(self) -> Consumer<T> {
        self.consumer
    }
}
