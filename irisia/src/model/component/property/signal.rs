use super::{PropCast, PropUpdate, Property, SignalPropAgent};
use crate::{Signal, model::component::property::PropertyAgent};

type Agent<T> = SignalPropAgent<Signal<T>>;

pub struct ThisPropertyIsRequiredButNotProvided;
type Missing = ThisPropertyIsRequiredButNotProvided;

impl<T: ?Sized> Property for Signal<T> {
    type Agent = Agent<T>;
    const __IRISIA_PROP_AGENT: Self::Agent = Agent::GET;
}

impl<T> PropertyAgent for Agent<T>
where
    T: ?Sized,
{
    type CastTarget = Signal<T>;

    type Empty = Missing;
    fn get_empty(&self) -> Self::Empty {
        Missing {}
    }
}

// allows any -> any
impl<T, Old> PropUpdate<Old, Missing> for Agent<T>
where
    T: ?Sized,
{
    type Output = Old;
    fn prop_update(&self, old: Old, _: Missing) -> Old {
        old
    }
}

// allow    undefined -> defined
// disallow defined   -> defined
impl<T> PropUpdate<Missing, Signal<T>> for Agent<T>
where
    T: ?Sized,
{
    type Output = Signal<T>;
    fn prop_update(&self, _: Missing, new: Signal<T>) -> Self::Output {
        new
    }
}

// value is provided, allow to cast
impl<T: ?Sized> PropCast<Signal<T>> for Agent<T> {
    fn prop_cast(&self, from: Signal<T>) -> Self::CastTarget {
        from
    }
}
