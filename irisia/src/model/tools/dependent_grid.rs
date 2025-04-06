use std::cell::RefCell;

use super::cursor::Cursor;

use super::dirty_set::{bitset_mark, bitset_union};

pub(super) struct DependentGrid {
    row_byte_width: usize,
    grid: RefCell<Vec<u8>>,
}

impl DependentGrid {
    pub fn new(width: usize) -> Self {
        let row_byte_width = width.div_ceil(8);
        Self {
            grid: vec![0u8; width * row_byte_width].into(),
            row_byte_width,
        }
    }

    pub fn expand_dep_tree(&mut self, dst: &mut [u8]) {
        let mut cursor = Cursor::new(0);
        while let Some(point) = cursor.next(dst) {
            let row = &mut self.grid.get_mut()
                [point * self.row_byte_width..(point + 1) * self.row_byte_width];
            bitset_union(dst, row);
            row.fill(0);
        }
    }

    pub(super) fn mark(&self, dependent: usize, dependency: usize) {
        let mut borrowed = self.grid.borrow_mut();
        let row =
            &mut borrowed[dependent * self.row_byte_width..(dependent + 1) * self.row_byte_width];
        bitset_mark(row, dependency);
    }
}
