/*
    This module provides a guarantee that the value wrapped in `ThreadGuard` can only
    be access by the thread where it was created, while `ThreadGuard` impls `Send` and
    `Sync`.
*/
use std::{
    ops::{Deref, DerefMut},
    thread::{self, ThreadId},
};

//pub type ThreadRc<T> = ThreadGuard<Rc<T>>;
//pub type ThreadWeak<T> = ThreadGuard<Weak<T>>;

#[doc(hidden)]
pub struct ThreadGuard<T> {
    tid: ThreadId,
    inner: T,
}

impl<T: Clone> Clone for ThreadGuard<T> {
    fn clone(&self) -> Self {
        self.assert_thread();
        ThreadGuard {
            tid: self.tid.clone(),
            inner: self.inner.clone(),
        }
    }
}

impl<T> ThreadGuard<T> {
    pub fn new(inner: T) -> Self {
        ThreadGuard {
            tid: thread::current().id(),
            inner,
        }
    }

    fn assert_thread(&self) {
        if thread::current().id() != self.tid {
            panic!("Cannot call from other thread");
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn into_inner(self) -> T {
        self.assert_thread();
        self.inner
    }
}

impl<T> Deref for ThreadGuard<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.assert_thread();
        &self.inner
    }
}

impl<T> DerefMut for ThreadGuard<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.assert_thread();
        &mut self.inner
    }
}

impl<T> From<T> for ThreadGuard<T> {
    #[inline]
    fn from(inner: T) -> Self {
        Self::new(inner)
    }
}

unsafe impl<T> Sync for ThreadGuard<T> {}
unsafe impl<T> Send for ThreadGuard<T> {}
