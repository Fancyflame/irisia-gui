use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

use super::StateUpdate;

// for String/OsString/PathBuf from str/OsStr/Path

macro_rules! impl_as_ref {
    ($Struct:ident $str:ident $push:ident) => {
        impl<T> StateUpdate<T> for $Struct
        where
            T: AsRef<$str>,
        {
            fn state_update(&mut self, updater: T, equality_matters: bool) -> bool {
                let unchanged = equality_matters && (self == updater.as_ref());
                self.clear();
                self.$push(updater.as_ref());
                unchanged
            }
        }
    };
}

impl_as_ref!(String   str   push_str);
impl_as_ref!(OsString OsStr push);
impl_as_ref!(PathBuf  Path  push);

// for Vec<T> from iterator

impl<I> StateUpdate<I> for Vec<I::Item>
where
    I: Iterator,
    I::Item: PartialEq<I::Item> + 'static,
{
    fn state_update(&mut self, mut updater: I, equaity_matters: bool) -> bool {
        if equaity_matters {
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
}

// for Box<T> from T's updater

impl<T, U> StateUpdate<U> for Box<T>
where
    T: StateUpdate<U> + ?Sized,
{
    fn state_update(&mut self, updater: U, equality_matters: bool) -> bool {
        (**self).state_update(updater, equality_matters)
    }
}
