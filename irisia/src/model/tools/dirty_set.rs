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

    pub fn data(&self) -> [u8; N] {
        self.0
    }

    pub fn union(&mut self, rhs: Self) {
        for (a, b) in self.0.iter_mut().zip(rhs.0.into_iter()) {
            *a |= b;
        }
    }
}

#[derive(Clone)]
pub struct Cursor {
    bytes: usize,
    bits: u32,
}

impl Cursor {
    pub(super) fn new(offset: usize) -> Self {
        Self {
            bytes: offset / 8,
            bits: (offset % 8) as _,
        }
    }

    pub(super) fn offset(&self) -> usize {
        self.bytes * 8 + self.bits as usize
    }

    pub fn next<'a>(&mut self, data: &'a [u8]) -> Option<usize> {
        let byte = loop {
            let mut byte = data.get(self.bytes).copied()?;
            byte &= !0 << self.bits;

            if byte == 0 {
                self.bytes += 1;
                self.bits = 0;
            } else {
                break byte;
            }
        };
        self.bits = byte.trailing_zeros();

        let result = self.offset();
        self.bits += 1;

        Some(result)
    }
}

#[test]
fn test() {
    let mut set = DirtySet::<5>::new();
    let inputs = [0, 10, 11, 39];
    for input in inputs {
        set.mark(input);
    }

    let mut cursor = Cursor::new(0);
    let data = set.data();
    let vec: Vec<usize> = std::iter::from_fn(|| cursor.next(&data)).collect();
    assert_eq!(&*vec, &inputs);
}
