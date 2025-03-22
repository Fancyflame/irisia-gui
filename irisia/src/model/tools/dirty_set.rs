use std::ops::BitOrAssign;

#[derive(Clone, Copy)]
pub struct DirtySet<const N: usize = 12>([u8; N]);

impl<const N: usize> DirtySet<N> {
    pub fn new() -> Self {
        Self([0; N])
    }

    pub fn mark(&mut self, position: usize) {
        let index = position / 8;
        let shifts = position % 8;
        let byte = self.0.get_mut(index).expect("position out of limit");
        *byte |= 1 << shifts;
    }

    pub fn iter(&self) -> DirtySetIter {
        let mut iter = self.0.iter();
        DirtySetIter {
            bits_offset: 0,
            current: iter.next().copied().unwrap_or(0),
            rest: iter,
        }
    }
}

#[derive(Clone)]
pub struct DirtySetIter<'a> {
    bits_offset: usize,
    current: u8,
    rest: std::slice::Iter<'a, u8>,
}

impl Iterator for DirtySetIter<'_> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        while self.current == 0 {
            self.bits_offset += 8;
            self.current = *self.rest.next()?;
        }

        let shifts = self.current.trailing_zeros();
        self.current &= !(1 << shifts);
        Some(self.bits_offset + shifts as usize)
    }
}

impl<const N: usize> BitOrAssign<Self> for DirtySet<N> {
    fn bitor_assign(&mut self, rhs: Self) {
        for (lhs, rhs) in self.0.iter_mut().zip(rhs.0.into_iter()) {
            *lhs |= rhs;
        }
    }
}

#[test]
fn test() {
    let mut set = DirtySet::<5>::new();
    let inputs = [0, 10, 11, 39];
    for input in inputs {
        set.mark(input);
    }
    let vec: Vec<usize> = set.iter().collect();
    assert_eq!(&*vec, &inputs);
}
