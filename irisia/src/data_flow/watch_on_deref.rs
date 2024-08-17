use std::{cell::OnceCell, ops::Deref};

use super::{ReadRef, Readable};

pub struct WatchOnDeref<'a, R>
where
    R: Readable + ?Sized,
{
    readable: &'a R,
    borrow: OnceCell<ReadRef<'a, R::Data>>,
}

impl<'a, R> WatchOnDeref<'a, R>
where
    R: Readable + ?Sized,
{
    pub fn new(readable: &'a R) -> Self {
        Self {
            readable,
            borrow: OnceCell::new(),
        }
    }

    pub fn inner(this: &Self) -> &R {
        &this.readable
    }

    pub fn clone_inner(this: &Self) -> R
    where
        R: Clone,
    {
        this.readable.clone()
    }
}

impl<'a, R> Deref for WatchOnDeref<'a, R>
where
    R: Readable + ?Sized,
{
    type Target = R::Data;

    fn deref(&self) -> &Self::Target {
        // we must call `read()` on each access,
        // because `read()` may trigger watching
        let r = self.readable.read();
        self.borrow.get_or_init(|| r)
    }
}
