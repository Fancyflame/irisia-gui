use std::rc::Rc;

use super::{Listener, Provider, Ref};

pub trait ProviderWrapper {
    type Data: ?Sized;

    fn inner(&self) -> &(impl Provider<Data = Self::Data> + ?Sized);
}

impl<T: ProviderWrapper> Provider for T {
    type Data = T::Data;

    fn read(&self) -> Ref<Self::Data> {
        self.inner().read()
    }

    fn dependent(&self, listener: Listener) {
        self.inner().dependent(listener);
    }
}
