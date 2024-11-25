use impl_variadics::impl_variadics;

use super::{Listener, Provider, Ref};

pub trait ProviderGroup {
    type DataWrapper<'a>
    where
        Self: 'a;

    type Data<'a>
    where
        Self: 'a;

    fn read_many(&self) -> Self::DataWrapper<'_>;
    fn deref_wrapper<'a, 'b>(wrapper: &'a Self::DataWrapper<'b>) -> Self::Data<'a>
    where
        Self: 'b;
    fn dependent_many(&self, _listener: Listener);
}

impl<T> ProviderGroup for T
where
    T: Provider,
{
    type DataWrapper<'a> = Ref<'a, T::Data>
    where
        Self: 'a;

    type Data<'a> = &'a <T as Provider>::Data
    where
        Self: 'a;

    fn read_many(&self) -> Self::DataWrapper<'_> {
        self.read()
    }

    fn deref_wrapper<'a, 'b>(wrapper: &'a Self::DataWrapper<'b>) -> Self::Data<'a>
    where
        Self: 'b,
    {
        &*wrapper
    }

    fn dependent_many(&self, listener: Listener) {
        self.dependent(listener);
    }
}

pub trait RefProviderGroup {
    type ToOwned: ProviderGroup;
    fn to_owned(self) -> Self::ToOwned;
}

impl_variadics! {
    ..=20 "T*" => {
        impl<#(#T0,)*> ProviderGroup for (#(#T0,)*)
        where
            #(#T0: ProviderGroup,)*
        {
            type DataWrapper<'a> = (#(<#T0 as ProviderGroup>::DataWrapper<'a>,)*)
            where
                Self: 'a;

            type Data<'a> = (#(<#T0 as ProviderGroup>::Data<'a>,)*)
            where
                Self: 'a;

            fn read_many(&self) -> Self::DataWrapper<'_> {
                (#(self.#index.read_many(),)*)
            }

            fn deref_wrapper<'a, 'b>(_wrapper: &'a Self::DataWrapper<'b>) -> Self::Data<'a>
            where
                Self: 'b
            {
                (#(<#T0 as ProviderGroup>::deref_wrapper(&_wrapper.#index),)*)
            }

            fn dependent_many(&self, _listener: Listener) {
                #(self.#index.dependent_many(_listener.clone());)*
            }
        }
    }
}
