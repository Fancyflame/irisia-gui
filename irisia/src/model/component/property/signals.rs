use crate::{
    Property, Signal,
    model::component::definition::{SignalProxied, UsingDefault},
};

use super::PropAssign;

pub struct ThereIsARequiredPropertyMissing;
pub type RequiredDefault = ThereIsARequiredPropertyMissing;

impl<T: ?Sized> Property for Signal<T> {
    type EmptyProp = RequiredDefault;
    const __IRISIA_EMPTY_PROP: Self::EmptyProp = RequiredDefault {};
}

impl<T: ?Sized> Property for Option<Signal<T>> {
    type EmptyProp = UsingDefault<T>;
    const __IRISIA_EMPTY_PROP: Self::EmptyProp = UsingDefault::GET;
}

impl<T, U> PropAssign<SignalProxied<T, U>> for Option<Signal<U>>
where
    T: Clone + 'static,
    U: ?Sized,
{
    fn prop_assign(value: Signal<U>) -> Self {
        Some(value)
    }
}
