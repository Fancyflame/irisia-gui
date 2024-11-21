use std::{ops::Deref, rc::Rc};

use super::{Provider, ProviderObject, Ref, ToProviderObject};

pub struct Constant<T>(Rc<dyn Provider<Data = T>>);

struct Inner<T>(T);

impl<T: 'static> Constant<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(Inner(value)))
    }
}

impl<T> Provider for Constant<T> {
    type Data = T;
    fn read(&self) -> Ref<Self::Data> {
        self.0.read()
    }
    fn dependent(&self, _: super::Listener) {}
}

impl<T> Provider for Inner<T> {
    type Data = T;
    fn read(&self) -> Ref<Self::Data> {
        Ref::Ref(&self.0)
    }
    fn dependent(&self, _: super::Listener) {}
}

impl<T> Deref for Constant<T> {
    type Target = dyn Provider<Data = T>;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<T> ToProviderObject for Constant<T> {
    type Data = T;
    fn to_object(&self) -> super::ProviderObject<Self::Data> {
        ProviderObject(self.0.clone())
    }
}
