use std::ops::Range;

use super::{Bitset, U32Array};

#[derive(Clone)]
pub struct DependencyIndexes<A: U32Array> {
    pub bitset: Bitset<A>,
}

impl<A: U32Array> DependencyIndexes<A> {
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

    pub(crate) fn union(&mut self, bitset: &Bitset<A>) {
        self.bitset.union(bitset);
    }

    pub(crate) fn skip_range(&mut self, range: Range<u32>) {
        self.bitset.clear_range(range)
    }
}
