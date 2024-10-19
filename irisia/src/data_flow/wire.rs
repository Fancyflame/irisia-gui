use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

use super::{
    deps::DepdencyList,
    trace_cell::{TraceCell, TraceRef},
    Listenable, Listener, ListenerList, ReadRef, ReadWire, Readable, ToListener, Wakeable,
};

const BORROW_ERROR: &str = "cannot update data inside the wire, because at least one reader still exists \
    or the last update operation has not completed. if it's because of the latter, it declares a wire \
    loop was detected, which means invoking the update function of this wire needs to read the \
    old data of this wire itself, which bound to cause infinite updating. to address this problem, \
    you should remove the loop manually";

pub fn wire<F, T, Mv>(f: F, helper_move: Mv) -> ReadWire<T>
where
    T: 'static,
    F: Fn(&Mv) -> T + 'static,
    Mv: 'static,
{
    Wire::new(move || {
        (f(&helper_move), move |r| {
            *r = f(&helper_move);
            true
        })
    })
}

pub fn wire_cmp<F, T, Mv>(f: F, helper_move: Mv) -> ReadWire<T>
where
    T: Eq + 'static,
    F: Fn(&Mv) -> T + 'static,
    Mv: 'static,
{
    Wire::new(move || {
        (f(&helper_move), move |r| {
            let value = f(&helper_move);
            let mutated = value != *r;
            *r = value;
            mutated
        })
    })
}

pub fn wire2<Fi, F, T>(init_state: Fi, f: F) -> ReadWire<T>
where
    T: 'static,
    Fi: FnOnce() -> T,
    F: Fn(Setter<T>) + 'static,
{
    wire3(move || (init_state(), f))
}

pub fn wire3<Fi, T, F>(fn_init: Fi) -> ReadWire<T>
where
    T: 'static,
    Fi: FnOnce() -> (T, F),
    F: Fn(Setter<T>) + 'static,
{
    Wire::new(move || {
        let (init, updater) = fn_init();
        (init, move |r| {
            let mut mutated = false;
            updater(Setter {
                r,
                mutated: &mut mutated,
            });
            mutated
        })
    })
}

struct Wire<F, T> {
    computes: TraceCell<Option<(F, T)>>,
    listeners: ListenerList,
    deps: DepdencyList,
    as_listenable: Weak<dyn Listenable>,
    is_dirty: Cell<bool>,
}

impl<F, T> Wire<F, T>
where
    F: Fn(&mut T) -> bool + 'static,
{
    fn new<Fi>(fn_init: Fi) -> Rc<Self>
    where
        Fi: FnOnce() -> (T, F),
        T: 'static,
    {
        let w = Rc::new_cyclic(move |this: &Weak<Wire<_, _>>| Wire {
            computes: TraceCell::new(None),
            listeners: ListenerList::new(),
            deps: DepdencyList::new(Listener::Weak(this.clone())),
            as_listenable: this.clone(),
            is_dirty: Cell::new(true),
        });

        let mut computes = w.computes.borrow_mut().unwrap();
        let init = w.deps.collect_dependencies(fn_init);
        *computes = Some((init.1, init.0));
        drop(computes);
        w
    }
}

impl<F, T> Readable for Wire<F, T>
where
    F: Fn(&mut T) -> bool,
{
    type Data = T;

    fn read(&self) -> ReadRef<Self::Data> {
        self.listeners
            .capture_caller(&self.as_listenable.upgrade().unwrap());
        ReadRef::CellRef(TraceRef::map(
            self.computes.borrow().expect(BORROW_ERROR),
            |computes| &computes.as_ref().unwrap().1,
        ))
    }

    fn ptr_as_id(&self) -> *const () {
        self.as_listenable.as_ptr().cast()
    }
}

impl<F, T> Wakeable for Wire<F, T>
where
    F: Fn(&mut T) -> bool,
{
    fn add_back_reference(&self, dep: &Rc<dyn Listenable>) {
        self.deps.add_dependency(dep);
    }

    fn set_dirty(&self) {
        self.is_dirty.set(true);
        self.listeners.set_dirty();
    }

    fn wake(&self) -> bool {
        if !self.is_dirty.get() {
            return true;
        }

        let mut computes_opt = self.computes.borrow_mut().expect(BORROW_ERROR);
        let (update_fn, state) = computes_opt.as_mut().unwrap();

        let mutated = self.deps.collect_dependencies(|| update_fn(state));
        if mutated {
            self.listeners.wake_all();
        }

        self.is_dirty.set(false);
        true
    }
}

impl<F, T> Listenable for Wire<F, T> {
    fn add_listener(&self, listener: &dyn ToListener) {
        self.listeners.add_listener(listener.to_listener());
    }

    fn remove_listener(&self, listener: &dyn ToListener) {
        self.listeners.remove_listener(listener.to_listener());
    }
}

pub struct Setter<'a, T: ?Sized> {
    r: &'a mut T,
    mutated: &'a mut bool,
}

impl<T: ?Sized> Deref for Setter<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.r
    }
}

impl<T: ?Sized> DerefMut for Setter<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        *self.mutated = true;
        self.r
    }
}
