use std::{cell::Cell, ops::Deref};

use crate::model::tools::{caller_stack, DirtySet};

pub struct FieldHook<T, const D_CAP: usize = 12> {
    dirty_set: Cell<DirtySet<D_CAP>>,
    check_eq: fn(&T, &T) -> bool,
    value: T,
}

impl<T, const D_CAP: usize> FieldHook<T, D_CAP> {
    pub fn new(value: T, check_eq: fn(&T, &T) -> bool) -> Self {
        Self {
            dirty_set: Cell::new(DirtySet::new()),
            check_eq,
            value,
        }
    }

    pub fn set_and_wake(this: &mut Self, value: T, target: &mut DirtySet<D_CAP>) {
        if (this.check_eq)(&value, &this.value) {
            return;
        }

        *target |= std::mem::replace(this.dirty_set.get_mut(), DirtySet::new());
        this.value = value;
    }
}

impl<T, const D_CAP: usize> Deref for FieldHook<T, D_CAP> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        if let Some(position) = caller_stack::get_caller() {
            let mut dirty_set = self.dirty_set.get();
            dirty_set.mark(position);
            self.dirty_set.set(dirty_set);
        }
        &self.value
    }
}
