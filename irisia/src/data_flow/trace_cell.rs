use std::{
    backtrace::Backtrace,
    cell::{Ref, RefCell, RefMut},
    collections::{hash_map::Entry, HashMap},
    fmt::Write,
    ops::{Deref, DerefMut},
};

use anyhow::Result;

pub struct TraceCell<T> {
    value: RefCell<T>,
    borrow_traces: RefCell<BorrowTraces>,
}

struct BorrowTraces {
    table: HashMap<u32, Backtrace>,
    next_id: u32,
}

impl<T> TraceCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: RefCell::new(value),
            borrow_traces: RefCell::new(BorrowTraces {
                table: HashMap::new(),
                next_id: 0,
            }),
        }
    }

    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }

    fn get_error(&self) -> anyhow::Error {
        let mut msg = "this cell has been occupied by: \n".to_string();
        for (index, bt) in self.borrow_traces.borrow().table.values().enumerate() {
            writeln!(&mut msg, "####### {index}: \n{}\n", bt).unwrap();
        }
        anyhow::Error::msg(msg)
    }

    pub fn borrow(&self) -> Result<TraceRef<Ref<T>>> {
        Ok(TraceRef {
            inner_ref: self.value.try_borrow().map_err(|_| self.get_error())?,
            trace: DropTrace::record(&self.borrow_traces),
        })
    }

    pub fn borrow_mut(&self) -> Result<TraceRef<RefMut<T>>> {
        Ok(TraceRef {
            inner_ref: self.value.try_borrow_mut().map_err(|_| self.get_error())?,
            trace: DropTrace::record(&self.borrow_traces),
        })
    }
}

struct DropTrace<'a> {
    trace_table: &'a RefCell<BorrowTraces>,
    id: u32,
}

impl<'a> DropTrace<'a> {
    fn record(borrow_traces: &'a RefCell<BorrowTraces>) -> Self {
        let backtrace = Backtrace::capture();
        let mut borrowed_r = borrow_traces.borrow_mut();
        let borrowed = &mut *borrowed_r;

        let id = loop {
            let this_id = borrowed.next_id;
            borrowed.next_id = borrowed.next_id.wrapping_add(1);
            if let Entry::Vacant(vac) = borrowed.table.entry(this_id) {
                vac.insert(backtrace);
                break this_id;
            }
        };

        Self {
            trace_table: &borrow_traces,
            id,
        }
    }
}

pub struct TraceRef<'a, T> {
    inner_ref: Ref<'a, T>,
    trace: DropTrace<'a>,
}

impl<'a, T: ?Sized> TraceRef<'a, T> {
    pub fn clone(this: &Self) -> Self {
        Self {
            inner_ref: Ref::clone(&this.inner_ref),
            trace: DropTrace::record(&this.trace.trace_table),
        }
    }

    pub fn map<U, F>(orig: Self, f: F) -> TraceRef<'a, Ref<'a, U>>
    where
        F: FnOnce(&T) -> &U,
        U: ?Sized,
    {
        TraceRef {
            inner_ref: Ref::map(orig.inner_ref, f),
            trace: orig.trace,
        }
    }
}

pub struct TraceMut<'a, T> {
    inner_ref: RefMut<'a, T>,
    trace: DropTrace<'a>,
}

impl<'a, T: ?Sized> TraceMut<'a, RefMut<'a, T>> {
    pub fn map<U, F>(orig: Self, f: F) -> TraceRef<'a, RefMut<'a, U>>
    where
        F: FnOnce(&mut T) -> &mut U,
        U: ?Sized,
    {
        TraceMut {
            inner_ref: RefMut::map(orig.inner_ref, f),
            trace: orig.trace,
        }
    }
}

impl<T: ?Sized> Deref for TraceRef<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner_ref
    }
}

impl<T: ?Sized> Deref for TraceMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner_ref
    }
}

impl<T: ?Sized> DerefMut for TraceMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner_ref
    }
}

impl Drop for DropTrace<'_> {
    fn drop(&mut self) {
        let _ = self
            .trace_table
            .borrow_mut()
            .table
            .remove(&self.id)
            .unwrap();
    }
}
