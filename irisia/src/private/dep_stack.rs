use std::{
    cell::RefCell,
    marker::PhantomData,
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

    pub fn scoped<T, F, R>(&self, target_ptr: &T, bitset: Bitset<WD>, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        debug_assert_eq!(bitset.iter().map(|x| x.count_ones()).sum::<u32>(), 1);
        self.stack.borrow_mut().push(DepInfo {
            self_ptr: target_ptr as *const T as _,
            bitset,
        });
        let r = f();
        self.stack.borrow_mut().pop();
        r
    }

    pub fn apply_dep<T>(&self, target_ptr: &T, bitset: &mut Bitset<WD>) {
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
