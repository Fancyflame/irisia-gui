use std::marker::PhantomData;

use crate::{style::StyleGroup, StyleReader, UpdateWith};

use super::*;

pub trait HelpCreate<S, T> {
    type Def: Defaulter<Src = S>;
    fn create(&self, maybe_init: T) -> Self::Def;
}

impl<S, T> HelpCreate<S, (T,)> for CallUpdater
where
    S: UpdateWith<T>,
{
    type Def = PropInitialized<S>;
    fn create(&self, maybe_init: (T,)) -> Self::Def {
        PropInitialized(S::create_with(maybe_init.0))
    }
}

impl<T> HelpCreate<T, (T,)> for MoveOwnership {
    type Def = PropInitialized<T>;
    fn create(&self, maybe_init: (T,)) -> Self::Def {
        PropInitialized(maybe_init.0)
    }
}

impl<S, T> HelpCreate<S, (T,)> for ReadStyle
where
    S: StyleReader,
    T: StyleGroup,
{
    type Def = PropInitialized<S>;
    fn create(&self, maybe_init: (T,)) -> Self::Def {
        PropInitialized(S::read_style(&maybe_init.0))
    }
}

impl<S, T> HelpCreate<S, (T,)> for fn(T) -> S {
    type Def = PropInitialized<S>;
    fn create(&self, maybe_init: (T,)) -> Self::Def {
        PropInitialized(self(maybe_init.0))
    }
}

impl<M, S> HelpCreate<S, ()> for M {
    type Def = PropNotInitialized<S>;
    fn create(&self, _: ()) -> Self::Def {
        PropNotInitialized(PhantomData)
    }
}
