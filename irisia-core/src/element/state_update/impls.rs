use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

use crate::{Style, StyleReader};

use super::{StateStore, StateUpdate};

// for String/OsString/PathBuf from str/OsStr/Path

macro_rules! impl_as_ref {
    ($Struct:ident $str:ident $push:ident) => {
        impl StateStore for $Struct {
            type Store = Self;
        }

        impl<T> StateUpdate<T> for $Struct
        where
            T: AsRef<$str>,
        {
            fn state_created(updater: T) -> Self {
                updater.as_ref().into()
            }

            fn state_changed(state: &mut Self, updater: T) {
                state.clear();
                state.$push(updater.as_ref());
            }
        }
    };
}

impl_as_ref!(String   str   push_str);
impl_as_ref!(OsString OsStr push);
impl_as_ref!(PathBuf  Path  push);

// for Vec<T> from iterator

impl<T: 'static> StateStore for Vec<T> {
    type Store = Self;
}

impl<I> StateUpdate<I> for Vec<I::Item>
where
    I: Iterator,
    I::Item: 'static,
{
    fn state_created(updater: I) -> Self {
        updater.collect()
    }

    fn state_changed(state: &mut Self, updater: I) {
        state.clear();
        state.extend(updater);
    }
}

// for any style reader from styles

pub struct StyleWatcher<T>(T);

impl<T> StateStore for StyleWatcher<T>
where
    T: 'static,
{
    type Store = T;
}

impl<T, S> StateUpdate<S> for StyleWatcher<T>
where
    T: StyleReader + 'static,
    S: Style,
{
    fn state_created(updater: S) -> T {
        T::read_style(&updater)
    }

    fn state_changed(state: &mut T, updater: S) {
        *state = T::read_style(&updater);
    }
}

// ---- Macro Automatically Call ----

// simply move the ownership

pub struct Move<T>(T);

impl<T: 'static> StateStore for Move<T> {
    type Store = T;
}

impl<T: 'static> StateUpdate<T> for Move<T> {
    fn state_created(updater: T) -> Self::Store {
        updater
    }

    fn state_changed(state: &mut Self::Store, updater: T) {
        *state = updater
    }
}

// for types initialize once, ignores state change

pub struct Once<T>(T);

impl<T: StateStore> StateStore for Once<T> {
    type Store = T::Store;
}

impl<T, U> StateUpdate<U> for Once<T>
where
    T: StateUpdate<U>,
{
    fn state_created(updater: U) -> Self::Store {
        T::state_created(updater)
    }

    fn state_changed(_: &mut Self::Store, _: U) {}
}

// for types could clone from any other types that reference to it

pub struct Cloned<T>(T);

impl<T: 'static> StateStore for Cloned<T> {
    type Store = T;
}

impl<T, U> StateUpdate<U> for Cloned<T>
where
    T: Clone + 'static,
    U: AsRef<T>,
{
    fn state_created(updater: U) -> Self::Store {
        updater.as_ref().clone()
    }

    fn state_changed(state: &mut Self::Store, updater: U) {
        *state = updater.as_ref().clone()
    }
}
