use std::marker::PhantomData;

pub mod macro_utils;
pub mod option_signal;
pub mod signal;

pub trait Property {
    type Agent: PropertyAgent<CastTarget = Self>;
    // const __IRISIA_PROP_AGENT: &'static Self::Agent;
    fn __irisia_prop_agent<'a>() -> &'a Self::Agent;
}

pub trait PropertyAgent {
    type CastTarget;

    type Empty;
    fn get_empty(&self) -> Self::Empty;
}

pub trait PropUpdate<T, U, SourceFrom = Self> {
    type Output;
    fn prop_update(&self, old: T, update: U) -> Self::Output;
}

pub trait PropCast<T>: PropertyAgent {
    fn prop_cast(&self, from: T) -> Self::CastTarget;
}

pub struct SignalPropAgent<T>(PhantomData<T>);

impl<T> SignalPropAgent<T> {
    const GET: Self = Self(PhantomData);
}
