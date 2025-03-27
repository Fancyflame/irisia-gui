use std::cell::RefCell;

use crate::model::{tools::Cursor, VModel};

use super::{mark_bit, watcher::Watcher, DirtySet};

pub struct DependentGrid {
    width: usize,
    grid: RefCell<Vec<u8>>,
}

impl DependentGrid {
    pub fn new<T>(_: &T) -> Self
    where
        T: VModel,
    {
        let width = T::EXECUTE_POINTS;
        Self {
            grid: vec![0u8; width * width].into(),
            width,
        }
    }

    pub fn flush_expand<const D_CAP: usize>(
        &mut self,
        input: &DirtySet<D_CAP>,
        dst: &mut DirtySet<D_CAP>,
    ) {
        assert_eq!(D_CAP, self.width);

        let mut new_ds: DirtySet<D_CAP> = DirtySet::new();
        new_ds.union(input.data());
        let mut cursor = Cursor::new(0);

        while let Some(point) = cursor.next(new_ds.data()) {
            dst.mark(point);
            let row: &mut [u8; D_CAP] = (&mut self.grid.get_mut()
                [point * D_CAP..(point + 1) * D_CAP])
                .try_into()
                .unwrap();
            new_ds.union(row);
            *row = [0u8; D_CAP];
        }
    }

    pub(crate) fn make_watcher<T>(&self, id: usize, value: T) -> Watcher<T> {
        Watcher::from_temp_var(self, id, value)
    }

    pub(super) fn mark(&self, dependent: usize, dependency: usize) {
        let mut borrowed = self.grid.borrow_mut();
        let row = &mut borrowed[dependent * self.width..(dependent + 1) * self.width];
        mark_bit(row, dependency);
    }
}
