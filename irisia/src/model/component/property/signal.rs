use super::{PropCast, PropUpdate, Property, SignalsCannotBeExtended};
use crate::Signal;

pub struct ThisPropertyIsRequiredButNotProvided;
type Missing = ThisPropertyIsRequiredButNotProvided;

impl<T: ?Sized> Property for Signal<T> {
    type Mutator = SignalsCannotBeExtended;
    const MUTATOR: Self::Mutator = SignalsCannotBeExtended;

    type Empty = Missing;
    const EMPTY: Self::Empty = Missing {};
}

// allows any -> any
impl<T, Old> PropUpdate<Old, Missing> for Signal<T>
where
    T: ?Sized,
{
    type Output = Old;
    fn prop_update(old: Old, _: Missing) -> Old {
        old
    }
}

// allow    undefined -> defined
// disallow defined   -> defined
impl<T> PropUpdate<Missing, Self> for Signal<T>
where
    T: ?Sized,
{
    type Output = Self;
    fn prop_update(_: Missing, new: Self) -> Self::Output {
        new
    }
}

// value is provided, allow to cast
impl<T: ?Sized> PropCast<Signal<T>> for Signal<T> {
    fn prop_cast(from: Signal<T>) -> Self {
        from
    }
}
