use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

/// Custom update methods.
pub trait UpdateFrom<T>: Sized {
    /// Update the state, returns whether
    /// the new state is equivalent to the previous.
    ///
    /// - `updater`: The new state.
    /// - `equality_matters`: Whether the return value is matters.
    /// If not, you can return `false` directly without checking the equality
    /// which will cost nothing more than `true`.
    /// - `return`: Whether the state has changed. Return `false` is always
    /// correct, but may cause unnecessary redrawing.
    fn state_update(&mut self, updater: T, equality_matters: bool) -> bool;
    fn state_create(updater: T) -> Self;
}

// for String/OsString/PathBuf from str/OsStr/Path

macro_rules! impl_as_ref {
    ($Struct:ident $slice:ident $push:ident) => {
        impl<T> UpdateFrom<T> for $Struct
        where
            T: AsRef<$slice>,
        {
            fn state_update(&mut self, updater: T, equality_matters: bool) -> bool {
                let unchanged = equality_matters && (self == updater.as_ref());
                self.clear();
                self.$push(updater.as_ref());
                unchanged
            }

            fn state_create(updater: T) -> Self {
                updater.as_ref().into()
            }
        }
    };
}

impl_as_ref!(String   str   push_str);
impl_as_ref!(OsString OsStr push);
impl_as_ref!(PathBuf  Path  push);

// for Vec<T> from iterator

impl<I> UpdateFrom<I> for Vec<I::Item>
where
    I: Iterator,
    I::Item: PartialEq<I::Item> + 'static,
{
    fn state_update(&mut self, mut updater: I, equality_matters: bool) -> bool {
        if equality_matters {
            let mut result = true;
            let mut old_elements = self.iter_mut();

            for (old, new) in (&mut old_elements).zip(&mut updater) {
                result = result && (*old == new);
                *old = new;
            }

            if old_elements.len() != 0 {
                result = false;
                let len = old_elements.len();
                self.drain(self.len() - len..);
            }

            if let Some(next) = updater.next() {
                result = false;
                self.push(next);
                self.extend(updater);
            }

            result
        } else {
            self.clear();
            self.extend(updater);
            false
        }
    }

    fn state_create(updater: I) -> Self {
        updater.collect()
    }
}

// for Box<T> from T's updater

impl<T, U> UpdateFrom<U> for Box<T>
where
    T: UpdateFrom<U> + ?Sized,
{
    fn state_update(&mut self, updater: U, equality_matters: bool) -> bool {
        (**self).state_update(updater, equality_matters)
    }

    fn state_create(updater: U) -> Self {
        Box::new(T::state_create(updater))
    }
}
