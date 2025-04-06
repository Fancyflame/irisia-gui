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

    pub fn next(&mut self, data: &[u8]) -> Option<usize> {
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

#[cfg(test)]
mod test {
    use super::Cursor;
    use crate::model::tools::dirty_set::{bitset_create, bitset_mark};

    #[test]
    fn test() {
        let mut bitset = bitset_create(4);
        let inputs = [0, 10, 11, 39];
        for input in inputs {
            bitset_mark(&mut bitset, input);
        }

        let mut cursor = Cursor::new(0);
        let vec: Vec<usize> = std::iter::from_fn(|| cursor.next(&bitset)).collect();
        assert_eq!(&*vec, &inputs);
    }
}
