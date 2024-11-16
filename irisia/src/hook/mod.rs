use std::rc::Rc;

pub use consumer::listener::Listener;

pub mod consumer;
pub mod listener_list;

pub trait Provider {
    fn dependent(&self, listener: Listener);
}

macro_rules! deref_impl {
    ($($Type: ty),*) => {
        $(
            impl<P> Provider for $Type
            where
                P: Provider + ?Sized,
            {
                fn dependent(&self, listener: Listener) {
                    (**self).dependent(listener);
                }
            }
        )*
    };
}

deref_impl!(&P, Rc<P>, Box<P>);
