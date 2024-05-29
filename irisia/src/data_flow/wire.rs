use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

use super::{
    convert_from::{Updater, UpdaterInner},
    Listener, ListenerList, ReadWire, Readable, Wakeable,
};

const BORROW_ERROR: &str = "cannot update data inside the wire, because the last update \
    operation has not completed. if you see this message it's probably because a wire loop \
    was detected, which means invoking the update function of this wire needs to read the \
    old data of this wire itself, which bound to cause infinite updating. to address this problem, \
    you should remove the loop manually";

struct Wire<F, T> {
    computes: RefCell<(F, T)>,

    // if value never changes, then we don't need a listener list
    listeners: Option<ListenerList>,
}

pub fn wire<F, R>(mut f: F) -> ReadWire<F::Output>
where
    F: FnMut() -> R + 'static,
    R: 'static,
{
    wire3(move || (f(), move |r| *r = f()))
}

pub fn wire2<T>(mut f: impl FnMut(Updater<'_, T>) + 'static) -> ReadWire<T>
where
    T: 'static,
{
    wire3(move || {
        let mut cache = None;
        f(Updater(UpdaterInner::Unassigned(&mut cache)));
        (cache.expect("not initialized"), move |r| {
            f(Updater(UpdaterInner::OutOfDate {
                target: r,
                updated: false,
            }))
        })
    })
}

pub fn wire3<F2, F, T>(f: F2) -> ReadWire<T>
where
    T: 'static,
    F2: FnOnce() -> (T, F),
    F: FnMut(&mut T) + 'static,
{
    let mut rc = Rc::new_cyclic(|weak: &Weak<Wire<F, T>>| {
        ListenerList::push_global_stack(Listener::Once(weak.clone()));
        let (data, update_fn) = f();
        ListenerList::pop_global_stack();

        Wire {
            computes: (update_fn, data).into(),
            listeners: Some(ListenerList::new()),
        }
    });

    if let Some(wire) = Rc::get_mut(&mut rc) {
        wire.listeners = None;
    }

    rc
}

impl<F, T> Readable for Wire<F, T> {
    type Data = T;

    fn read(&self) -> Ref<Self::Data> {
        self.listeners.as_ref().map(ListenerList::capture_caller);
        Ref::map(self.computes.borrow(), |(_, cache)| cache)
    }

    fn pipe(&self, listen_end: Listener) {
        self.listeners.as_ref().map(|ll| ll.watch(&listen_end));
    }
}

impl<F, T> Wakeable for Wire<F, T>
where
    T: 'static,
    F: FnMut(&mut T) + 'static,
{
    fn update(self: Rc<Self>) -> bool {
        ListenerList::push_global_stack(Listener::Once(Rc::downgrade(&self) as _));

        let mut computes_ref = self.computes.try_borrow_mut().expect(BORROW_ERROR);
        let computes = &mut *computes_ref;

        (computes.0)(&mut computes.1);
        drop(computes_ref);

        ListenerList::pop_global_stack();
        self.listeners.as_ref().map(ListenerList::wake_all);
        true
    }
}
