use impl_variadics::impl_variadics;

use super::{Listener, Provider, Ref};

pub trait ProviderGroup {
    type Data<'a>
    where
        Self: 'a;
    fn read_many(&self) -> Self::Data<'_>;
    fn dependent_many(&self, listener: Listener);
}

impl<T> ProviderGroup for T
where
    T: Provider,
{
    type Data<'a> = Ref<'a, T::Data>
    where
        Self: 'a;

    fn read_many(&self) -> Self::Data<'_> {
        self.read()
    }

    fn dependent_many(&self, listener: Listener) {
        self.dependent(listener);
    }
}

impl_variadics! {
    ..=20 "T*" => {
        impl<#(#T0,)*> ProviderGroup for (#(#T0,)*)
        where
            #(#T0: ProviderGroup,)*
        {
            type Data<'a> = (#(<#T0 as ProviderGroup>::Data<'a>,)*)
            where
                Self: 'a;

            fn read_many(&self) -> Self::Data<'_> {
                (#(self.#index.read_many(),)*)
            }

            fn dependent_many(&self, _listener: Listener) {
                #(self.#index.dependent_many(_listener.clone());)*
            }
        }
    }
}
