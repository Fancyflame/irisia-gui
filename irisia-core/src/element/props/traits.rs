use std::marker::PhantomData;

use crate::{style::StyleContainer, StyleReader};

use super::UpdateFrom;

pub struct CallUpdater;
pub struct MoveOwnership;
pub struct ReadStyle;

// update

pub trait HelpUpdate<S, T>: HelpCreate<S, T> {
    fn update(&self, source: &mut S, maybe_init: T, equality_matters: bool) -> bool;
}

impl<S, T> HelpUpdate<S, (T,)> for CallUpdater
where
    S: UpdateFrom<T>,
{
    fn update(&self, source: &mut S, maybe_init: (T,), equality_matters: bool) -> bool {
        source.state_update(maybe_init.0, equality_matters)
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

// defaulter

pub trait Defaulter {
    type Src;

    fn with_defaulter<F>(self, defaulter: F) -> Self::Src
    where
        F: FnOnce() -> Self::Src;
}

#[derive(Clone, Copy)]
pub struct NeedDefaulter<S>(PhantomData<S>);

impl<S> Defaulter for NeedDefaulter<S> {
    type Src = S;
    fn with_defaulter<F>(self, defaulter: F) -> Self::Src
    where
        F: FnOnce() -> Self::Src,
    {
        defaulter()
    }
}

pub struct MustBeInitialized<S>(pub S);

impl<S> Defaulter for MustBeInitialized<S> {
    type Src = S;
    fn with_defaulter<F>(self, _: F) -> Self::Src
    where
        F: FnOnce() -> Self::Src,
    {
        self.0
    }
}

impl<S> MustBeInitialized<S> {
    pub fn must_be_initialized(self) -> S {
        self.0
    }
}

// create

pub trait HelpCreate<S, T> {
    type Def: Defaulter<Src = S>;
    fn create(&self, maybe_init: T) -> Self::Def;
}

impl<S, T> HelpCreate<S, (T,)> for CallUpdater
where
    S: UpdateFrom<T>,
{
    type Def = MustBeInitialized<S>;
    fn create(&self, maybe_init: (T,)) -> Self::Def {
        MustBeInitialized(S::state_create(maybe_init.0))
    }
}

impl<T> HelpCreate<T, (T,)> for MoveOwnership {
    type Def = MustBeInitialized<T>;
    fn create(&self, maybe_init: (T,)) -> Self::Def {
        MustBeInitialized(maybe_init.0)
    }
}

impl<S, T> HelpCreate<S, (T,)> for ReadStyle
where
    S: StyleReader,
    T: StyleContainer,
{
    type Def = MustBeInitialized<S>;
    fn create(&self, maybe_init: (T,)) -> Self::Def {
        MustBeInitialized(S::read_style(&maybe_init.0))
    }
}

impl<S, T> HelpCreate<S, (T,)> for fn(T) -> S {
    type Def = MustBeInitialized<S>;
    fn create(&self, maybe_init: (T,)) -> Self::Def {
        MustBeInitialized(self(maybe_init.0))
    }
}

impl<M, S> HelpCreate<S, ()> for M {
    type Def = NeedDefaulter<S>;
    fn create(&self, _: ()) -> Self::Def {
        NeedDefaulter(PhantomData)
    }
}
