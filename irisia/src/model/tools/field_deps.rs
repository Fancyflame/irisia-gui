use std::cell::RefCell;

use super::dirty_set::{bitset_mark, bitset_union};

pub struct FieldDeps {
    row_byte_width: usize,
    grid: RefCell<Vec<u8>>,
}

impl FieldDeps {
    pub fn new(rows: usize, row_width: usize) -> Self {
        let row_byte_width = row_width.div_ceil(8);
        Self {
            row_byte_width,
            grid: vec![0; rows * row_byte_width].into(),
        }
    }

    pub fn mark(&self, src: usize, dep_id: usize) {
        bitset_mark(
            get_row(&mut self.grid.borrow_mut(), self.row_byte_width, src),
            dep_id,
        );
    }

    pub fn take(&mut self, dst: &mut [u8], src: usize) {
        let row = get_row(self.grid.get_mut(), self.row_byte_width, src);
        bitset_union(dst, &row);
        row.fill(0);
    }
}

fn get_row(all: &mut [u8], rw: usize, src: usize) -> &mut [u8] {
    &mut all[src * rw..(src + 1) * rw]
}
