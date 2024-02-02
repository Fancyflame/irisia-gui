use std::{
    cell::{Cell, RefCell},
    ops::{Deref, DerefMut, Range},
};

// DependentStack

#[derive(Default)]
pub struct DependentStack<const WD: usize> {
    stack: RefCell<Vec<u32>>,
}

impl<const WD: usize> DependentStack<WD> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scoped<F, R>(&self, caller_id: u32, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.stack.borrow_mut().push(caller_id);
        let r = f();
        self.stack.borrow_mut().pop();
        r
    }

    pub(crate) fn collect_dep(&self, mut bitset: Bitset<WD>) -> Bitset<WD> {
        if let Some(caller_id) = self.stack.borrow().last().copied() {
            bitset.set(caller_id);
        }
        bitset
    }
}

// DataSource

pub struct DataSource<T, const WD: usize> {
    data: T,
    bitset: Cell<Bitset<WD>>,
}

impl<T, const WD: usize> DataSource<T, WD> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            bitset: Cell::new(Bitset::new()),
        }
    }

    pub fn read(&self, stack: &DependentStack<WD>) -> &T {
        self.bitset.set(stack.collect_dep(self.bitset.get()));
        &self.data
    }

    pub fn update(&mut self) -> (&mut T, DependencyIndexes<WD>) {
        (&mut self.data, self.bitset.take().dependency_indexes())
    }
}

// Bitset

#[derive(Clone, Copy)]
pub struct Bitset<const WD: usize>([usize; WD]);

impl<const WD: usize> Default for Bitset<WD> {
    fn default() -> Self {
        Bitset([0; WD])
    }
}

impl<const WD: usize> Deref for Bitset<WD> {
    type Target = [usize; WD];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const WD: usize> DerefMut for Bitset<WD> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const WD: usize> Bitset<WD> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, position: u32) {
        self[(position / usize::BITS) as usize] |= 1 << (position % usize::BITS);
    }

    pub fn dependency_indexes(&self) -> DependencyIndexes<WD> {
        DependencyIndexes { bitset: *self }
    }

    pub fn union(&mut self, other: &Self) {
        for (c1, c2) in self.iter_mut().zip(other.into_iter()) {
            *c1 |= c2;
        }
    }

    pub fn clear_range(&mut self, range: Range<u32>) {
        let Range { start, end } = range;
        debug_assert!(start <= end);

        let (start_chunk, end_chunk) = (start / usize::BITS, end / usize::BITS);
        let (start_bit_skip, end_bit_skip) = (start % usize::BITS, end % usize::BITS);

        for chunk in start_chunk..=end_chunk {
            let mut mask_neg = usize::MAX;

            if chunk == start_chunk {
                mask_neg <<= start_bit_skip;
            }

            if chunk == end_chunk {
                mask_neg <<= end_bit_skip;
                mask_neg >>= end_bit_skip;
            }

            self[chunk as usize] &= !mask_neg;
        }
    }
}

// DependencyIndexes

#[derive(Clone)]
pub struct DependencyIndexes<const WD: usize> {
    bitset: Bitset<WD>,
}

impl<const WD: usize> DependencyIndexes<WD> {
    pub fn step_if(&mut self, expected: u32) -> bool {
        for (index, bits) in self.bitset.iter_mut().enumerate() {
            if *bits == 0 {
                continue;
            }

            let offset = bits.trailing_zeros();
            let next = offset + index as u32 * usize::BITS;

            if next == expected {
                *bits &= !(1 << offset); // set that 1 to 0
                return true;
            }
        }
        false
    }

    pub(crate) fn peek(&self) -> Option<u32> {
        for (index, bits) in self.bitset.iter().enumerate() {
            if *bits == 0 {
                continue;
            }

            let offset = bits.trailing_zeros();
            return Some(offset + index as u32 * usize::BITS);
        }
        None
    }

    pub(crate) fn union(&mut self, bitset: &Bitset<WD>) {
        self.bitset.union(bitset);
    }

    pub(crate) fn skip_range(&mut self, range: Range<u32>) {
        self.bitset.clear_range(range)
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

        let stack: DependentStack<2> = DependentStack::new();

        let biggest = usize::BITS * 2 - 1;
        let middle = usize::BITS * 3 / 2;
        let smallest = usize::BITS / 3;

        let mut src = DataSource::new(0);

        stack.scoped(biggest, || {
            src.read(&stack);
        });

        stack.scoped(smallest, || {
            src.read(&stack);
        });

        stack.scoped(middle, || {
            src.read(&stack);
        });

        let (_, mut di) = src.update();
        assert!(di.step_if(smallest));
        assert!(di.step_if(middle));
        assert!(di.step_if(biggest));
        assert!(di.peek().is_none());
    }
}
