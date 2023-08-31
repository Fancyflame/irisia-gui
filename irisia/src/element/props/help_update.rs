use crate::{style::StyleContainer, StyleReader, UpdateWith};

use super::*;

pub trait HelpUpdate<S, T>: HelpCreate<S, T> {
    fn update(&self, source: &mut S, maybe_init: T, equality_matters: bool) -> bool;
}

impl<S, T> HelpUpdate<S, (T,)> for CallUpdater
where
    S: UpdateWith<T>,
{
    fn update(&self, source: &mut S, maybe_init: (T,), equality_matters: bool) -> bool {
        source.update_with(maybe_init.0, equality_matters)
    }
}

impl<T> HelpUpdate<T, (T,)> for MoveOwnership
where
    T: PartialEq<T>,
{
    fn update(&self, source: &mut T, maybe_init: (T,), equality_matters: bool) -> bool {
        let eq = equality_matters && *source == maybe_init.0;
        *source = maybe_init.0;
        eq
    }
}

impl<S, T> HelpUpdate<S, (T,)> for ReadStyle
where
    S: StyleReader + PartialEq<S>,
    T: StyleContainer,
{
    fn update(&self, source: &mut S, maybe_init: (T,), equality_matters: bool) -> bool {
        let style = S::read_style(&maybe_init.0);
        let eq = equality_matters && style == *source;
        *source = style;
        eq
    }
}

impl<S, T> HelpUpdate<S, (T,)> for fn(T) -> S
where
    S: PartialEq<S>,
{
    fn update(&self, source: &mut S, maybe_init: (T,), equality_matters: bool) -> bool {
        let value = self(maybe_init.0);
        let eq = equality_matters && *source == value;
        *source = value;
        eq
    }
}

impl<M, S> HelpUpdate<S, ()> for M {
    fn update(&self, _: &mut S, _: (), equality_matters: bool) -> bool {
        equality_matters
    }
}
