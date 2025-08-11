use impl_variadics::impl_variadics;

use super::{utils::trace_cell::TraceRef, Listener, Signal};

pub trait SignalGroup {
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

impl<T> SignalGroup for Signal<T>
where
    T: ?Sized,
{
    type DataWrapper<'a>
        = TraceRef<'a, T>
    where
        Self: 'a;

    type Data<'a>
        = &'a T
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

impl<T> SignalGroup for Option<T>
where
    T: SignalGroup,
{
    type DataWrapper<'a>
        = Option<T::DataWrapper<'a>>
    where
        Self: 'a;

    type Data<'a>
        = Option<T::Data<'a>>
    where
        Self: 'a;

    fn read_many(&self) -> Self::DataWrapper<'_> {
        self.as_ref().map(|v| v.read_many())
    }

    fn deref_wrapper<'a, 'b>(wrapper: &'a Self::DataWrapper<'b>) -> Self::Data<'a>
    where
        Self: 'b,
    {
        wrapper.as_ref().map(|v| T::deref_wrapper(v))
    }

    fn dependent_many(&self, listener: Listener) {
        if let Some(value) = self {
            value.dependent_many(listener);
        }
    }
}

pub trait RefProviderGroup {
    type ToOwned: SignalGroup;
    fn to_owned(self) -> Self::ToOwned;
}

impl_variadics! {
    ..=20 "T*" => {
        impl<#(#T0,)*> SignalGroup for (#(#T0,)*)
        where
            #(#T0: SignalGroup,)*
        {
            type DataWrapper<'a> = (#(<#T0 as SignalGroup>::DataWrapper<'a>,)*)
            where
                Self: 'a;

            type Data<'a> = (#(<#T0 as SignalGroup>::Data<'a>,)*)
            where
                Self: 'a;

            fn read_many(&self) -> Self::DataWrapper<'_> {
                (#(self.#index.read_many(),)*)
            }

            fn deref_wrapper<'a, 'b>(_wrapper: &'a Self::DataWrapper<'b>) -> Self::Data<'a>
            where
                Self: 'b
            {
                (#(<#T0 as SignalGroup>::deref_wrapper(&_wrapper.#index),)*)
            }

            fn dependent_many(&self, _listener: Listener) {
                #(self.#index.dependent_many(_listener.clone());)*
            }
        }
    }
}
