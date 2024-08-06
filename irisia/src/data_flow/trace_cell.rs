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
    table: HashMap<usize, Backtrace>,
    next_id: usize,
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

    fn get_error(&self) -> anyhow::Error {
        let mut msg = "this cell has been occupied by: \n".to_string();
        for (index, bt) in self.borrow_traces.borrow().table.values().enumerate() {
            writeln!(&mut msg, "####### {index}: \n{}\n", bt).unwrap();
        }
        anyhow::Error::msg(msg)
    }

    pub fn borrow(&self, bt: Backtrace) -> Result<TraceRef<Ref<T>>> {
        Ok(TraceRef {
            inner_ref: self.value.try_borrow().map_err(|_| self.get_error())?,
            trace: DropTrace::record(&self.borrow_traces, bt),
        })
    }

    pub fn borrow_mut(&self, bt: Backtrace) -> Result<TraceRef<RefMut<T>>> {
        Ok(TraceRef {
            inner_ref: self.value.try_borrow_mut().map_err(|_| self.get_error())?,
            trace: DropTrace::record(&self.borrow_traces, bt),
        })
    }
}

struct DropTrace<'a> {
    trace_table: &'a RefCell<BorrowTraces>,
    id: usize,
}

impl<'a> DropTrace<'a> {
    fn record(borrow_traces: &'a RefCell<BorrowTraces>, bt: Backtrace) -> Self {
        let mut borrowed_r = borrow_traces.borrow_mut();
        let borrowed = &mut *borrowed_r;

        let id = loop {
            let this_id = borrowed.next_id;
            borrowed.next_id = borrowed.next_id.wrapping_add(1);
            if let Entry::Vacant(vac) = borrowed.table.entry(this_id) {
                vac.insert(bt);
                break this_id;
            }
        };

        Self {
            trace_table: &borrow_traces,
            id,
        }
    }
}

pub struct TraceRef<'a, R> {
    inner_ref: R,
    trace: DropTrace<'a>,
}

impl<'a, T: ?Sized> TraceRef<'a, Ref<'a, T>> {
    pub fn clone(this: &Self) -> Self {
        let bt = Backtrace::force_capture();
        Self {
            inner_ref: Ref::clone(&this.inner_ref),
            trace: DropTrace::record(&this.trace.trace_table, bt),
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

impl<'a, T: ?Sized> TraceRef<'a, RefMut<'a, T>> {
    pub fn map_mut<U, F>(orig: Self, f: F) -> TraceRef<'a, RefMut<'a, U>>
    where
        F: FnOnce(&mut T) -> &mut U,
        U: ?Sized,
    {
        TraceRef {
            inner_ref: RefMut::map(orig.inner_ref, f),
            trace: orig.trace,
        }
    }
}

impl<'a, R> Deref for TraceRef<'a, R>
where
    R: Deref,
{
    type Target = R::Target;

    fn deref(&self) -> &Self::Target {
        &self.inner_ref
    }
}

impl<'a, R> DerefMut for TraceRef<'a, R>
where
    R: DerefMut,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
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
