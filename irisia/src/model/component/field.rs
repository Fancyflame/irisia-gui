use std::{cell::Cell, ops::Deref};

use crate::model::{dirty_set::DirtySet, UPDATE_POINT_STACK};

pub struct AutoField<T, const D_CAP: usize = 12> {
    dirty_set: Cell<DirtySet<D_CAP>>,
    value: T,
}

impl<T, const D_CAP: usize> AutoField<T, D_CAP> {
    pub fn new(value: T) -> Self {
        Self {
            dirty_set: Cell::new(DirtySet::new()),
            value,
        }
    }
}

impl<T, const D_CAP: usize> Deref for AutoField<T, D_CAP> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        UPDATE_POINT_STACK.with_borrow(|vec| {
            if let Some(&position) = vec.last() {
                let mut set = self.dirty_set.get();
                set.mark(position);
                self.dirty_set.set(set);
            }
        });
        &self.value
    }
}
