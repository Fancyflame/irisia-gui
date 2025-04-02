use impl_variadics::impl_variadics;

use crate::coerce_signal;

use super::{Provider, ProviderObject, Ref, Signal, ToProviderObject};

#[derive(Clone)]
pub enum SimpleProvider<T> {
    Owned(T),
    Hook(ProviderObject<T>),
}

impl<T> Provider for SimpleProvider<T> {
    type Data = T;
    fn read(&self) -> Ref<Self::Data> {
        match self {
            Self::Owned(v) => Ref::Ref(v),
            Self::Hook(h) => h.read(),
        }
    }
    fn dependent(&self, listener: super::Listener) {
        match self {
            Self::Owned(_) => {}
            Self::Hook(h) => h.dependent(listener),
        }
    }
}

impl<T> SimpleProvider<T> {
    pub fn new<U, _M>(value: U) -> Self
    where
        Self: MarkedCast<U, _M>,
    {
        Self::marked_cast(value)
    }

    pub fn into_object(self) -> ProviderObject<T>
    where
        T: 'static,
    {
        match self {
            Self::Hook(h) => h,
            Self::Owned(v) => Signal::state(v).to_object(),
        }
    }
}

impl<T> Default for SimpleProvider<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::Owned(Default::default())
    }
}

pub trait MarkedCast<T, _M> {
    fn marked_cast(value: T) -> Self;
}

pub struct AsSelf;
impl<T> MarkedCast<T, AsSelf> for SimpleProvider<T> {
    fn marked_cast(value: T) -> Self {
        Self::Owned(value)
    }
}

pub struct AsHook;
impl<T> MarkedCast<T, AsHook> for SimpleProvider<T::Data>
where
    T: ToProviderObject,
    T::Data: Sized,
{
    fn marked_cast(value: T) -> Self {
        Self::Hook(value.to_object())
    }
}

pub struct AsFunc<Args>(Args);
impl_variadics! {
    ..=3 "Arg*" => {
        impl<F, #(#Arg0,)*> MarkedCast<F, AsFunc<(#(#Arg0,)*)>> for ProviderObject<dyn Fn(#(#Arg0,)*)>
        where
            F: Fn(#(#Arg0,)*) + 'static,
        {
            fn marked_cast(value: F) -> Self {
                let sig: Signal<dyn Fn(#(#Arg0,)*)> = coerce_signal!(&Signal::state(value));
                sig.to_object()
            }
        }
    }
}
