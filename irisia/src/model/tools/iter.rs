use super::{
    caller_stack,
    cursor::Cursor,
    watcher::{Watcher, WatcherType},
    DepManager,
};

pub struct DirtyPoints<'a> {
    pub(super) cursor: Cursor,
    pub(super) mgr: &'a DepManager,
}

impl<'a> DirtyPoints<'a> {
    pub(super) fn new(dep_mgr: &'a DepManager) -> Self {
        Self {
            cursor: Cursor::new(0),
            mgr: dep_mgr,
        }
    }

    pub fn check_range(&self, upper_bound: usize) -> bool {
        let peeked = self.cursor.clone().next(&self.mgr.current_dp);
        match peeked {
            Some(p) => p < self.offset() + upper_bound,
            None => false,
        }
    }

    pub fn consume(&mut self, upper_bound: usize) {
        self.cursor = Cursor::new(self.cursor.offset() + upper_bound);
    }

    pub fn offset(&self) -> usize {
        self.cursor.offset()
    }

    pub(crate) fn fork(&self) -> Self {
        DirtyPoints {
            cursor: self.cursor.clone(),
            mgr: self.mgr,
        }
    }

    pub fn watch_field<T>(&self, src: usize) {
        if let Some(caller) = caller_stack::get_caller() {
            self.mgr.field_deps.mark(src, caller);
        }
    }

    pub fn watch_temp_var<T>(&self, exec_point: usize, value: T) -> Watcher<'a, T> {
        Watcher {
            wt: WatcherType::TempVar {
                grid: &self.mgr.grid,
                exec_point,
            },
            value,
        }
    }
}
