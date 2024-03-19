use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use smallvec::SmallVec;

use super::{bitset::U32Array, Bitset, DependencyIndexes};

pub struct DependentStack<A: U32Array>(Rc<Inner<A>>);

struct Inner<A: U32Array> {
    stack: RefCell<SmallVec<[u32; 2]>>,
    dirty_set: Cell<Bitset<A>>,
}

impl<A: U32Array> DependentStack<A> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scoped<F, R>(&self, caller_id: u32, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.0.stack.borrow_mut().push(caller_id);
        let r = f();
        self.0.stack.borrow_mut().pop();
        r
    }

    pub fn share(&self) -> Self {
        DependentStack(self.0.clone())
    }

    pub(crate) fn add_dirty(&self, bitset: &Bitset<A>) {
        let mut new = self.0.dirty_set.get();
        new.union(&bitset);
        self.0.dirty_set.set(new);
    }

    pub(crate) fn collect_dep(&self, mut bitset: Bitset<A>) -> Bitset<A> {
        if let Some(caller_id) = self.0.stack.borrow().last().copied() {
            bitset.set(caller_id);
        }
        bitset
    }

    pub(crate) fn get_update_list(&self, clear: bool) -> DependencyIndexes<A> {
        if clear {
            self.0.dirty_set.take()
        } else {
            self.0.dirty_set.get()
        }
        .dependency_indexes()
    }
}

impl<A: U32Array> Default for DependentStack<A> {
    fn default() -> Self {
        Self(Rc::new(Inner {
            stack: Default::default(),
            dirty_set: Default::default(),
        }))
    }
}