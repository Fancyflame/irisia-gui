use std::cell::Cell;

use super::{bitset::UsizeArray, Bitset, DependentStack};

pub struct DataSource<T, A: UsizeArray> {
    data: T,
    bitset: Cell<Bitset<A>>,
    dep_stack: DependentStack<A>,
}

impl<T, A: UsizeArray> DataSource<T, A> {
    pub fn new(data: T, dep_stack: &DependentStack<A>) -> Self {
        Self {
            data,
            bitset: Cell::new(Bitset::new()),
            dep_stack: dep_stack.share(),
        }
    }

    pub fn read(&self) -> &T {
        self.bitset
            .set(self.dep_stack.collect_dep(self.bitset.get()));
        &self.data
    }

    pub fn update(&mut self) -> &mut T {
        self.dep_stack.add_dirty(&self.bitset.take());
        &mut self.data
    }
}

pub trait CreateBuilder {
    type Builder: Default;
}

pub trait CreateByBuilder<A, B>
where
    A: UsizeArray,
{
    fn create_by_builder(builder: B, dep_stack: &DependentStack<A>) -> Self;
}
