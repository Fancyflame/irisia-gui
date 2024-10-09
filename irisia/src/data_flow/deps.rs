use std::{
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    rc::{Rc, Weak},
};

use smallvec::SmallVec;

use super::{Listenable, Wakeable};

thread_local! {
    static STACK: RefCell<Vec<Listener>> = Default::default();
}

type ListenerTable = HashMap<*const (), Listener>;

#[derive(Default)]
pub(super) struct ListenerList {
    listeners: RefCell<ListenerTable>,
    delay_operation: RefCell<SmallVec<[Operation; 2]>>,
}

enum Operation {
    Add(Listener),

    // should not store `*const ()` here,
    // we cannot ensure that address was not be reused
    Remove(Weak<dyn Wakeable>),
}

impl ListenerList {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn capture_caller(&self, dep: &Rc<dyn Listenable>) {
        STACK.with_borrow(|vec| {
            if let Some(listener) = vec.last() {
                self.add_listener(&listener.clone());
                listener.assume_valid().add_back_reference(dep);
            }
        });
    }

    pub fn add_listener(&self, listener: &Listener) {
        self.try_operate(Operation::Add(listener.clone()));
    }

    pub fn remove_listener(&self, listener: &Listener) {
        self.try_operate(Operation::Remove(match listener {
            Listener::Rc(rc) => Rc::downgrade(rc),
            Listener::Weak(weak) => weak.clone(),
        }));
    }

    fn try_operate(&self, opr: Operation) {
        match self.listeners.try_borrow_mut() {
            Ok(mut list) => {
                Self::operate(&mut list, opr);
            }
            Err(_) => self.delay_operation.borrow_mut().push(opr),
        }
    }

    fn operate(listeners: &mut ListenerTable, opr: Operation) {
        match opr {
            Operation::Add(listener) => {
                let key: *const () = match listener {
                    Listener::Rc(rc) => Rc::as_ptr(&rc) as _,
                    Listener::Weak(weak) => weak.as_ptr() as _,
                };
                listeners.insert(key, listener);
            }
            Operation::Remove(weak) => {
                listeners.remove(&(weak.as_ptr() as _));
            }
        }
    }

    fn for_each_listeners<F>(table: &mut ListenerTable, mut f: F)
    where
        F: FnMut(Rc<dyn Wakeable>),
    {
        table.retain(|_, listener| match listener.optioned() {
            Some(l) => {
                f(l);
                true
            }
            None => false,
        });
    }

    pub fn set_dirty(&self) {
        Self::for_each_listeners(&mut self.listeners.borrow_mut(), |listener| {
            listener.set_dirty();
        });
    }

    pub fn wake_all(&self) {
        if !self.is_dirty.get() {
            return;
        }

        self.is_dirty.set(false);
        let mut listeners = self.listeners.borrow_mut();

        Self::for_each_listeners(&mut listeners, |listener| {
            listener.wake();
        });

        let mut delay_operation = self.delay_operation.borrow_mut();
        for opr in delay_operation.drain(..) {
            Self::operate(&mut listeners, opr);
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty.get()
    }
}

pub struct DepdencyList {
    dependencies: RefCell<HashMap<*const (), (Rc<dyn Listenable>, bool)>>,
    self_as_listener: Listener,
}

impl DepdencyList {
    pub fn new(listener: Listener) -> Self {
        Self {
            dependencies: RefCell::new(HashMap::new()),
            self_as_listener: listener,
        }
    }

    pub fn add_dependency(&self, dep: &Rc<dyn Listenable>) {
        match self.dependencies.borrow_mut().entry(Rc::as_ptr(dep) as _) {
            Entry::Occupied(mut occ) => {
                occ.get_mut().1 = true;
            }
            Entry::Vacant(vac) => {
                vac.insert((dep.clone(), true));
            }
        }
    }

    pub fn collect_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        for (_, state) in self.dependencies.borrow_mut().values_mut() {
            *state = false;
        }

        STACK.with_borrow_mut(|vec| vec.push(self.self_as_listener.clone()));
        let res = f();
        STACK.with_borrow_mut(|vec| vec.pop());

        self.dependencies.borrow_mut().retain(|_, (dep, used)| {
            if !*used {
                dep.remove_listener(&self.self_as_listener.clone());
            }
            *used
        });

        res
    }

    pub fn clear(&self) {
        for (_, (dep, _)) in self.dependencies.borrow_mut().drain() {
            dep.remove_listener(&self.self_as_listener);
        }
    }
}

#[derive(Clone)]
pub enum Listener {
    Rc(Rc<dyn Wakeable>),
    Weak(Weak<dyn Wakeable>),
}

impl Listener {
    fn optioned(&self) -> Rc<dyn Wakeable> {
        match self {
            Listener::Rc(l) => Some(l.clone()),
            Listener::Weak(l) => l.upgrade(),
        }
    }
}
