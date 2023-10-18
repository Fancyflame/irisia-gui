use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
};

#[derive(Default)]
pub struct DependentStack<const WD: usize> {
    stack: RefCell<Vec<DepInfo<WD>>>,
}

struct DepInfo<const WD: usize> {
    self_ptr: *const (),
    bitset: Bitset<WD>,
}

impl<const WD: usize> DependentStack<WD> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scoped<T, F, R>(&self, target_ptr: &T, bitset: &Bitset<WD>, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        debug_assert_eq!(bitset.iter().map(|x| x.count_ones()).sum::<u32>(), 1);
        self.stack.borrow_mut().push(DepInfo {
            self_ptr: target_ptr as *const T as _,
            bitset: bitset.clone(),
        });
        let r = f();
        self.stack.borrow_mut().pop();
        r
    }

    pub fn get_dep<T>(&self, target_ptr: &T, bitset: &mut Bitset<WD>) {
        let borrowed = self.stack.borrow();
        let src = match borrowed.last() {
            Some(info) if info.self_ptr == target_ptr as *const T as *const () => &info.bitset,
            _ => return,
        };

        for index in 0..WD {
            bitset[index] |= src[index];
        }
    }
}

#[derive(Clone)]
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

    pub fn new_with_one_setted(index: u32) -> Self {
        let mut this = Self::new();
        let chunk = &mut this.0[(index / usize::BITS) as usize];
        *chunk |= 1 << (index % usize::BITS);
        this
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

        let target = ();
        let biggest = usize::BITS * 2 - 1;
        let middle = usize::BITS * 3 / 2;
        let smallest = usize::BITS / 3;

        let mut dep_bitset = Bitset::<2>::new();

        stack.scoped(&target, &Bitset::new_with_one_setted(biggest), || {
            stack.get_dep(&target, &mut dep_bitset);
        });

        stack.scoped(&target, &Bitset::new_with_one_setted(smallest), || {
            stack.get_dep(&target, &mut dep_bitset);
        });

        stack.scoped(&target, &Bitset::new_with_one_setted(middle), || {
            stack.get_dep(&target, &mut dep_bitset);
        });

        // target is not the same, not captured
        stack.scoped(&(), &Bitset::new_with_one_setted(0), || {
            stack.get_dep(&target, &mut dep_bitset);
        });

        let mut iter = dep_bitset.dependency_indexes();
        assert_eq!(iter.next(), Some(smallest));
        assert_eq!(iter.next(), Some(middle));
        assert_eq!(iter.next(), Some(biggest));
        assert_eq!(iter.next(), None);
    }
}
