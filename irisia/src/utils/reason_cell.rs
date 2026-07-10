#![allow(dead_code)]

use std::cell::{Cell, Ref, RefCell, RefMut};

const TEMP_BORROW_REASON: &str =
    "temporary ReasonCell borrow still be held, should DROP IMMEDIATELY after use";

pub struct ReasonCell<T: ?Sized> {
    reason: Cell<&'static str>,
    value: RefCell<T>,
}

impl<T> ReasonCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            reason: Cell::new("unreachable"),
            value: RefCell::new(value),
        }
    }

    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
}

impl<T: ?Sized> ReasonCell<T> {
    pub fn borrow(&self, lock_reason: &'static str) -> Ref<'_, T> {
        let borrowed = self
            .value
            .try_borrow()
            .unwrap_or_else(|_| panic!("{}", self.reason.get()));
        self.reason.set(lock_reason);
        borrowed
    }

    pub fn borrow_mut(&self, lock_reason: &'static str) -> RefMut<'_, T> {
        let borrowed = self
            .value
            .try_borrow_mut()
            .unwrap_or_else(|_| panic!("{}", self.reason.get()));
        self.reason.set(lock_reason);
        borrowed
    }

    pub fn temp_borrow(&self) -> Ref<'_, T> {
        self.borrow(TEMP_BORROW_REASON)
    }

    pub fn temp_borrow_mut(&self) -> RefMut<'_, T> {
        self.borrow_mut(TEMP_BORROW_REASON)
    }
}

impl<T> From<T> for ReasonCell<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}
