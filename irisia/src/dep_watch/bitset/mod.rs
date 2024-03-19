use std::ops::{Deref, DerefMut, Range};

pub use self::dependency_indexes::DependencyIndexes;

mod dependency_indexes;

#[derive(Clone, Copy)]
pub struct Bitset<A>(A);

impl<A: U32Array> Default for Bitset<A> {
    fn default() -> Self {
        Bitset(A::zeroed())
    }
}

impl<A: U32Array> Deref for Bitset<A> {
    type Target = [usize];
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<A: U32Array> DerefMut for Bitset<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut_slice()
    }
}

impl<A: U32Array> Bitset<A> {
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

pub trait U32Array: Sized + AsRef<[u32]> + AsMut<[u32]> + Copy + 'static {
    fn zeroed() -> Self
    where
        Self: Sized;
}

impl<const N: usize> U32Array for [u32; N] {
    fn zeroed() -> Self {
        [0; N]
    }
}
