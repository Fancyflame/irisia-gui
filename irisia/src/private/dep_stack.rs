use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

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

    pub fn collect_dep(&self, bitset: &mut Bitset<WD>) {
        let Some(caller_id) = self.stack.borrow().last().copied()
        else {
            return;
        };

        bitset[(caller_id / usize::BITS) as usize] |= 1 << (caller_id % usize::BITS);
    }
}

#[derive(Clone, Copy)]
pub struct Bitset<const WD: usize>(pub [usize; WD]);

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
        Bitset([0; WD])
    }

    pub fn dependency_indexes(&self) -> impl Iterator<Item = u32> {
        self.0
            .into_iter()
            .enumerate()
            .map(|(index, mut bits)| {
                std::iter::from_fn(move || {
                    if bits == 0 {
                        return None;
                    }
                    let offset = bits.trailing_zeros();
                    bits &= !(1 << offset); // set that 1 to 0
                    Some(offset + index as u32 * usize::BITS)
                })
            })
            .flatten()
    }

    pub fn clear(&mut self) {
        self.0 = [0; WD];
    }
}

pub const fn bitset_width(field_count: u32) -> usize {
    let mut width = field_count / usize::BITS;
    if field_count % usize::BITS != 0 {
        width += 1;
    }
    width as usize
}

#[cfg(test)]
mod test {
    use super::{Bitset, DependentStack};

    #[test]
    fn test_bitset() {
        assert!(usize::BITS >= 8);

        let stack: DependentStack<2> = DependentStack::new();

        let biggest = usize::BITS * 2 - 1;
        let middle = usize::BITS * 3 / 2;
        let smallest = usize::BITS / 3;

        let mut dep_bitset = Bitset::<2>::new();

        stack.scoped(biggest, || {
            stack.collect_dep(&mut dep_bitset);
        });

        stack.scoped(smallest, || {
            stack.collect_dep(&mut dep_bitset);
        });

        stack.scoped(middle, || {
            stack.collect_dep(&mut dep_bitset);
        });

        let mut iter = dep_bitset.dependency_indexes();
        assert_eq!(iter.next(), Some(smallest));
        assert_eq!(iter.next(), Some(middle));
        assert_eq!(iter.next(), Some(biggest));
        assert_eq!(iter.next(), None);
    }
}
