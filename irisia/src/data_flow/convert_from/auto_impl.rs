use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

use smallvec::{Array, SmallVec};

use super::ConvertFrom;

// for String/OsString/PathBuf from str/OsStr/Path

macro_rules! impl_as_ref {
    ($Struct:ident $slice:ident $push:ident) => {
        impl<T> ConvertFrom<T> for $Struct
        where
            T: AsRef<$slice>,
        {
            fn update_from(&mut self, value: T) {
                self.clear();
                self.$push(value.as_ref());
            }

            fn create_from(value: T) -> Self {
                value.as_ref().into()
            }
        }
    };
}

impl_as_ref!(String   str   push_str);
impl_as_ref!(OsString OsStr push);
impl_as_ref!(PathBuf  Path  push);

// for Vec<T> from iterator

macro_rules! impl_vec {
    ($Vec:ty) => {
        fn update_from(&mut self, value: I) {
            self.clear();
            self.extend(value);
        }

        fn create_from(value: I) -> Self {
            value.collect()
        }
    };
}

impl<I> ConvertFrom<I> for Vec<I::Item>
where
    I: Iterator,
    I::Item: PartialEq<I::Item> + 'static,
{
    impl_vec!(Vec);
}

impl<I, A> ConvertFrom<I> for SmallVec<A>
where
    A: Array,
    A::Item: PartialEq<I::Item> + 'static,
    I: Iterator<Item = A::Item>,
{
    impl_vec!(SmallVec<A>);
}

// for Box<T> from T's updater

impl<T, U> ConvertFrom<U> for Box<T>
where
    T: ConvertFrom<U>,
{
    fn update_from(&mut self, value: U) {
        (**self).update_from(value);
    }

    fn create_from(value: U) -> Self {
        Box::new(T::create_from(value))
    }
}
