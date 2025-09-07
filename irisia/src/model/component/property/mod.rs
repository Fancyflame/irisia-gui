use crate::Signal;

pub trait Property {
    type Mutator;
    const MUTATOR: Self::Mutator;

    type Empty;
    const EMPTY: Self::Empty;
}

pub trait PropUpdate<T, U, SourceFrom = Self> {
    type Output;
    fn prop_update(old: T, update: U) -> Self::Output;
}

pub trait PropCast<T> {
    fn prop_cast(from: T) -> Self;
}

pub struct SignalsCannotBeExtended;

// Implementations for Signal<T>

pub struct ThisPropertyIsRequiredButNotProvided;
type Missing = ThisPropertyIsRequiredButNotProvided;

impl<T: ?Sized> Property for Signal<T> {
    type Mutator = SignalsCannotBeExtended;
    const MUTATOR: Self::Mutator = SignalsCannotBeExtended;

    type Empty = Missing;
    const EMPTY: Self::Empty = Missing {};
}

impl<T, Old> PropUpdate<Old, Missing> for Signal<T>
where
    T: ?Sized,
{
    type Output = Old;
    fn prop_update(old: Old, _: Missing) -> Old {
        old
    }
}

impl<T, Old> PropUpdate<Old, Self> for Signal<T>
where
    T: ?Sized,
{
    type Output = Self;
    fn prop_update(_: Old, new: Self) -> Self::Output {
        new
    }
}

impl<T: ?Sized> PropCast<Signal<T>> for Signal<T> {
    fn prop_cast(from: Signal<T>) -> Self {
        from
    }
}

// Implementations for Option<Signal<T>>

impl<T: ?Sized> Property for Option<Signal<T>> {
    type Mutator = SignalsCannotBeExtended;
    const MUTATOR: Self::Mutator = SignalsCannotBeExtended;

    type Empty = ();
    const EMPTY: Self::Empty = ();
}

impl<T, Old> PropUpdate<Old, ()> for Option<Signal<T>>
where
    T: ?Sized,
{
    type Output = Old;
    fn prop_update(old: Old, _: ()) -> Old {
        old
    }
}

impl<T, Old> PropUpdate<Old, Signal<T>> for Option<Signal<T>>
where
    T: ?Sized,
{
    type Output = Signal<T>;
    fn prop_update(_: Old, value: Signal<T>) -> Self::Output {
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
