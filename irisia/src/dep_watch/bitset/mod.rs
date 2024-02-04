use std::ops::{Deref, DerefMut, Range};

pub use self::dependency_indexes::DependencyIndexes;

mod dependency_indexes;

#[derive(Clone, Copy)]
pub struct Bitset<A: UsizeArray>(A);

impl<A: UsizeArray> Default for Bitset<A> {
    fn default() -> Self {
        Bitset(A::zeroed())
    }
}

impl<A: UsizeArray> Deref for Bitset<A> {
    type Target = [usize];
    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl<A: UsizeArray> DerefMut for Bitset<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut_slice()
    }
}

impl<A: UsizeArray> Bitset<A> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, position: u32) {
        self[(position / usize::BITS) as usize] |= 1 << (position % usize::BITS);
    }

    pub fn dependency_indexes(&self) -> DependencyIndexes<A> {
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

pub trait UsizeArray: Sized + Copy + 'static {
    fn as_slice(&self) -> &[usize];
    fn as_mut_slice(&mut self) -> &mut [usize];
    fn zeroed() -> Self;
}

impl<const N: usize> UsizeArray for [usize; N] {
    fn as_slice(&self) -> &[usize] {
        self
    }

    fn as_mut_slice(&mut self) -> &mut [usize] {
        self
    }

    fn zeroed() -> Self
    where
        Self: Sized,
    {
        [0; N]
    }
}
