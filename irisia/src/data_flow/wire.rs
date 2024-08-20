use std::{
    backtrace::Backtrace,
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

use super::{
    const_data::const_wire,
    convert_from::{Updater, UpdaterInner},
    trace_cell::{TraceCell, TraceRef},
    Listener, ListenerList, ReadRef, ReadWire, Readable, Wakeable,
};

const BORROW_ERROR: &str = "cannot update data inside the wire, because at least one reader still exists \
    or the last update operation has not completed. if it's because of the latter, it declares a wire \
    loop was detected, which means invoking the update function of this wire needs to read the \
    old data of this wire itself, which bound to cause infinite updating. to address this problem, \
    you should remove the loop manually";

struct Wire<F, T> {
    computes: TraceCell<(F, T)>,
    listeners: ListenerList,
}

pub fn wire<F, R>(mut f: F) -> ReadWire<F::Output>
where
    F: FnMut() -> R + 'static,
    R: 'static,
{
    wire3(move || (f(), move |mut r| *r = f()), false)
}

pub fn wire2<T>(mut f: impl FnMut(Updater<'_, T>) + 'static) -> ReadWire<T>
where
    T: 'static,
{
    wire3(
        move || {
            let mut cache = None;
            f(Updater(UpdaterInner::Unassigned(&mut cache)));
            (cache.expect("not initialized"), move |mut r| {
                f(Updater(UpdaterInner::OutOfDate {
                    target: &mut r,
                    updated: false,
                }))
            })
        },
        false,
    )
}

pub fn wire3<F2, F, T>(f: F2, update_immediately: bool) -> ReadWire<T>
where
    T: 'static,
    F2: FnOnce() -> (T, F),
    F: FnMut(WireMutator<T>) + 'static,
{
    let rc = Rc::new_cyclic(|weak: &Weak<Wire<F, T>>| {
        ListenerList::push_global_stack(Listener::Weak(weak.clone()));
        let (mut data, mut update_fn) = f();
        if update_immediately {
            update_fn(WireMutator {
                r: &mut data,
                mutated: &mut true,
            });
        }
        ListenerList::pop_global_stack();

        Wire {
            computes: TraceCell::new((update_fn, data)),
            listeners: ListenerList::new(),
        }
    });

    // We assume that if the rc was not taken, the wire will never be waked.
    if Rc::weak_count(&rc) == 0 && Rc::strong_count(&rc) == 1 {
        let value = Rc::into_inner(rc).unwrap().computes.into_inner().1;
        return const_wire(value);
    }

    rc
}

impl<F, T> Readable for Wire<F, T> {
    type Data = T;

    fn read(&self) -> ReadRef<Self::Data> {
        let bt = Backtrace::force_capture();
        self.listeners.capture_caller();
        ReadRef::CellRef(TraceRef::map(
            self.computes.borrow(bt).unwrap(),
            |(_, cache)| cache,
        ))
    }

    fn pipe(&self, listen_end: Listener) {
        self.listeners.watch(&listen_end);
    }
}

impl<F, T> Wakeable for Wire<F, T>
where
    T: 'static,
    F: FnMut(WireMutator<T>) + 'static,
{
    fn update(self: Rc<Self>) -> bool {
        ListenerList::push_global_stack(Listener::Weak(Rc::downgrade(&self) as _));

        let mut computes_ref = self
            .computes
            .borrow_mut(Backtrace::force_capture())
            .expect(BORROW_ERROR);
        let computes = &mut *computes_ref;

        let mut mutated = false;
        (computes.0)(WireMutator {
            r: &mut computes.1,
            mutated: &mut mutated,
        });
        drop(computes_ref);

        ListenerList::pop_global_stack();

        if mutated {
            self.listeners.wake_all();
        }

        true
    }
}

pub struct WireMutator<'a, T> {
    r: &'a mut T,
    mutated: &'a mut bool,
}

impl<T> Deref for WireMutator<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.r
    }
}

impl<T> DerefMut for WireMutator<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        *self.mutated = true;
        self.r
    }
}
