use super::{constant::Constant, Provider, ProviderObject, Ref, ToProviderObject};

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
        U: IntoSimpleProvider<T, _M>,
    {
        value.into_simple_provider()
    }

    pub fn into_object(self) -> ProviderObject<T>
    where
        T: 'static,
    {
        match self {
            Self::Hook(h) => h,
            Self::Owned(v) => Constant::new(v).to_object(),
        }
    }
}

pub trait IntoSimpleProvider<T, _M> {
    fn into_simple_provider(self) -> SimpleProvider<T>;
}

pub struct AsValue;
impl<T> IntoSimpleProvider<T, AsValue> for T {
    fn into_simple_provider(self) -> SimpleProvider<T> {
        SimpleProvider::Owned(self)
    }
}

pub struct AsHook;
impl<T> IntoSimpleProvider<T::Data, AsHook> for T
where
    T: ToProviderObject,
{
    fn into_simple_provider(self) -> SimpleProvider<T::Data> {
        SimpleProvider::Hook(self.to_object())
    }
}
