use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

use smallvec::{Array, SmallVec};

pub trait SpecificUpdate {
    type UpdateTo;
}

/// Custom update methods.
pub trait UpdateWith<T>: Sized {
    /// Update the state, returns whether
    /// the new state is equivalent to the previous.
    ///
    /// - `updater`: The new state.
    /// - `equality_matters`: Whether the return value is matters.
    /// If not, you can return `false` directly without checking the equality
    /// which will cost nothing more than `true`.
    /// - `return`: Whether the state has changed. Return `false` is always
    /// correct, but may cause unnecessary redrawing.
    fn update_with(&mut self, updater: T, equality_matters: bool) -> bool;
    fn create_with(updater: T) -> Self;

    fn update_option(option: Option<Self>, updater: T, equality_matters: &mut bool) -> Self {
        match option {
            Some(mut this) => {
                *equality_matters &= this.update_with(updater, *equality_matters);
                this
            }
            None => {
                *equality_matters = false;
                Self::create_with(updater)
            }
        }
    }
}

// for String/OsString/PathBuf from str/OsStr/Path

macro_rules! impl_as_ref {
    ($Struct:ident $slice:ident $push:ident) => {
        impl<T> UpdateWith<T> for $Struct
        where
            T: AsRef<$slice>,
        {
            fn update_with(&mut self, updater: T, equality_matters: bool) -> bool {
                let unchanged = equality_matters && (self == updater.as_ref());
                self.clear();
                self.$push(updater.as_ref());
                unchanged
            }

            fn create_with(updater: T) -> Self {
                updater.as_ref().into()
            }
        }
    };
}

impl_as_ref!(String   str   push_str);
impl_as_ref!(OsString OsStr push);
impl_as_ref!(PathBuf  Path  push);

// for Vec<T> from iterator

macro_rules! impl_vec {
    ($Vec:ty, $I: ident) => {
        fn update_with(&mut self, mut updater: $I, equality_matters: bool) -> bool {
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

        fn create_with(updater: $I) -> Self {
            updater.collect()
        }
    };
}

impl<I> UpdateWith<I> for Vec<I::Item>
where
    I: Iterator,
    I::Item: PartialEq<I::Item> + 'static,
{
    impl_vec!(Vec, I);
}

impl<I, A> UpdateWith<I> for SmallVec<A>
where
    A: Array,
    A::Item: PartialEq<I::Item> + 'static,
    I: Iterator<Item = A::Item>,
{
    impl_vec!(SmallVec<A>, I);
}

// for Box<T> from T's updater

impl<T, U> UpdateWith<U> for Box<T>
where
    T: UpdateWith<U> + ?Sized,
{
    fn update_with(&mut self, updater: U, equality_matters: bool) -> bool {
        (**self).update_with(updater, equality_matters)
    }

    fn create_with(updater: U) -> Self {
        Box::new(T::create_with(updater))
    }
}
