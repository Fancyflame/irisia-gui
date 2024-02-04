use std::cell::Cell;

use self::bitset::UsizeArray;
pub use self::{
    bitset::{Bitset, DependencyIndexes},
    dep_stack::DependentStack,
};

pub mod bitset;
pub mod dep_stack;

// DataSource

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

    pub fn update(&mut self) -> (&mut T, DependencyIndexes<A>) {
        (&mut self.data, self.bitset.take().dependency_indexes())
    }
}

// Utils

pub const fn bitset_width(field_count: u32) -> usize {
    let mut width = field_count / usize::BITS;
    if field_count % usize::BITS != 0 {
        width += 1;
    }
    width as usize
}

#[cfg(test)]
mod test {
    use super::{DataSource, DependentStack};

    #[test]
    fn test_bitset() {
        assert!(usize::BITS >= 8);

        let stack: DependentStack<[usize; 2]> = DependentStack::new();

        let biggest = usize::BITS * 2 - 1;
        let middle = usize::BITS * 3 / 2;
        let smallest = usize::BITS / 3;

        let mut src = DataSource::new(0, &stack);

        stack.scoped(biggest, || {
            src.read();
        });

        stack.scoped(smallest, || {
            src.read();
        });

        stack.scoped(middle, || {
            src.read();
        });

        let (_, mut di) = src.update();
        assert!(di.step_if(smallest));
        assert!(di.step_if(middle));
        assert!(di.step_if(biggest));
        assert!(di.peek().is_none());
    }
}
