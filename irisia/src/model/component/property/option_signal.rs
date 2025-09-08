use super::{PropCast, PropUpdate, Property, SignalsCannotBeExtended};
use crate::Signal;

impl<T: ?Sized> Property for Option<Signal<T>> {
    type Mutator = SignalsCannotBeExtended;
    const MUTATOR: Self::Mutator = SignalsCannotBeExtended;

    type Empty = ();
    const EMPTY: Self::Empty = ();
}

// allow any -> any
impl<T, Old> PropUpdate<Old, ()> for Option<Signal<T>>
where
    T: ?Sized,
{
    type Output = Old;
    fn prop_update(old: Old, _: ()) -> Old {
        old
    }
}

// allow    undefined -> defined
// disallow defined   -> defined
impl<T> PropUpdate<(), Signal<T>> for Option<Signal<T>>
where
    T: ?Sized,
{
    type Output = Signal<T>;
    fn prop_update(_: (), value: Signal<T>) -> Self::Output {
        value
    }
}

impl<T: ?Sized> PropCast<Signal<T>> for Option<Signal<T>> {
    fn prop_cast(from: Signal<T>) -> Self {
        Some(from)
    }
}

impl<T: ?Sized> PropCast<()> for Option<Signal<T>> {
    fn prop_cast(_: ()) -> Self {
        None
    }
}
