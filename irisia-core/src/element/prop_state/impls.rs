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
            fn state_update(state: &mut Self, updater: T) {
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
    fn state_update(state: &mut Self, updater: I) {
        state.clear();
        state.extend(updater);
    }
}

// for any style reader from styles

#[derive(Default)]
pub struct StyleWatcher<T>(T);

impl<T> StateStore for StyleWatcher<T>
where
    T: Default + 'static,
{
    type Store = T;
}

impl<T, S> StateUpdate<S> for StyleWatcher<T>
where
    T: StyleReader + Default + 'static,
    S: Style,
{
    fn state_update(state: &mut T, updater: S) {
        *state = T::read_style(&updater);
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
    fn state_update(_: &mut Self::Store, _: U) {}
}
