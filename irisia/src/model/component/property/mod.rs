pub mod option_signal;
pub mod signal;

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
