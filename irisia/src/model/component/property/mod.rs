use crate::{
    Signal,
    model::component::property::type_option::{TNone, TSome},
};

pub mod type_option;

pub trait MergePropertiesFrom<Src> {
    type Output;
    fn merge(self, src: Src) -> Self::Output;
}

pub trait PropertyMutator {
    const GET: Self;
}

pub trait PropFrom<T> {
    fn prop_from(from: T) -> Self;
}

impl<T> PropFrom<TSome<Signal<T>>> for Signal<T> {
    fn prop_from(from: TSome<Signal<T>>) -> Self {
        from.0
    }
}

impl<T> PropFrom<TSome<Signal<T>>> for Option<Signal<T>> {
    fn prop_from(from: TSome<Signal<T>>) -> Self {
        Some(from.0)
    }
}

impl<T> PropFrom<TNone> for Option<Signal<T>> {
    fn prop_from(_: TNone) -> Self {
        None
    }
}
