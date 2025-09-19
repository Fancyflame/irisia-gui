use super::{PropCast, PropUpdate, Property, SignalPropAgent};
use crate::{
    Signal,
    model::component::{definition::UsingDefault, property::PropertyAgent},
};

type Agent<T> = SignalPropAgent<Option<Signal<T>>>;

impl<T: ?Sized> Property for Option<Signal<T>> {
    type Agent = Agent<T>;
    fn __irisia_prop_agent<'a>() -> &'a Self::Agent
    where
        Self::Agent: 'a,
    {
        &Agent::GET
    }
}

impl<T> PropertyAgent for Agent<T>
where
    T: ?Sized,
{
    type CastTarget = Option<Signal<T>>;

    type Empty = UsingDefault<T>;
    fn get_empty(&self) -> Self::Empty {
        UsingDefault::GET
    }
}

// allow any -> any
impl<T, Old> PropUpdate<Old, ()> for Agent<T>
where
    T: ?Sized,
{
    type Output = Old;
    fn prop_update(&self, old: Old, _: ()) -> Old {
        old
    }
}

// allow    undefined -> defined
// disallow defined   -> defined
impl<T> PropUpdate<(), Signal<T>> for Agent<T>
where
    T: ?Sized,
{
    type Output = Signal<T>;
    fn prop_update(&self, _: (), value: Signal<T>) -> Self::Output {
        value
    }
}

impl<T> PropUpdate<(), Option<Signal<T>>> for Agent<T>
where
    T: ?Sized,
{
    type Output = Option<Signal<T>>;
    fn prop_update(&self, _: (), value: Option<Signal<T>>) -> Self::Output {
        value
    }
}

impl<T: ?Sized> PropCast<Signal<T>> for Agent<T> {
    fn prop_cast(&self, from: Signal<T>) -> Self::CastTarget {
        Some(from)
    }
}

impl<T: ?Sized> PropCast<Option<Signal<T>>> for Agent<T> {
    fn prop_cast(&self, from: Option<Signal<T>>) -> Self::CastTarget {
        from
    }
}

impl<T: ?Sized> PropCast<()> for Agent<T> {
    fn prop_cast(&self, _: ()) -> Self::CastTarget {
        None
    }
}
