use std::{cell::UnsafeCell, thread::ThreadId};

pub struct ThreadGuard<T: ?Sized> {
    thread_id: ThreadId,
    value: UnsafeCell<T>,
}

impl<T: ?Sized> ThreadGuard<T> {}
