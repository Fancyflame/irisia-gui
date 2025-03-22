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
        DirtySetIter {
            data: &self.0,
            cursor_byte: 0,
            cursor_bits: 0,
        }
    }
}

impl<const N: usize> BitOrAssign<Self> for DirtySet<N> {
    fn bitor_assign(&mut self, rhs: Self) {
        for (lhs, rhs) in self.0.iter_mut().zip(rhs.0.into_iter()) {
            *lhs |= rhs;
        }
    }
}

#[derive(Clone)]
pub struct DirtySetIter<'a> {
    cursor_byte: usize,
    cursor_bits: u32,
    data: &'a [u8],
}

impl Iterator for DirtySetIter<'_> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        let byte = loop {
            let mut byte = self.data.get(self.cursor_byte).copied()?;
            byte &= !0 << self.cursor_bits;

            if byte == 0 {
                self.cursor_byte += 1;
                self.cursor_bits = 0;
            } else {
                break byte;
            }
        };

        let shifts = byte.trailing_zeros();
        self.cursor_bits = shifts + 1;

        Some(self.cursor_byte * 8 + shifts as usize)
    }
}

impl DirtySetIter<'_> {
    pub fn new_empty() -> Self {
        Self {
            cursor_byte: 0,
            cursor_bits: 0,
            data: &[],
        }
    }

    pub fn peek(&self) -> Option<usize> {
        self.clone().next()
    }

    pub fn set_cursor(&mut self, new_position: usize) {
        self.cursor_byte = new_position % 8;
        self.cursor_bits = (new_position / 8) as _;
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
