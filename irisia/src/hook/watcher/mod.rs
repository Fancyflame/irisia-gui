use std::{
    any::type_name,
    cell::{Cell, RefCell},
    rc::Rc,
};

use smallvec::SmallVec;

use super::{Listener, listener::StrongListener, signal_group::SignalGroup};

pub struct WatcherList(SmallVec<[Watcher; 1]>);

impl WatcherList {
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

impl WatcherGuard for WatcherList {
    fn push(&mut self, watcher: Watcher) {
        self.0.push(watcher);
    }
}

pub struct Watcher(#[allow(unused)] StrongListener);

pub trait WatcherGuard {
    fn push(&mut self, watcher: Watcher);

    // Provided

    fn watch<F, Return, D>(&mut self, callback: F, deps: D) -> &mut Self
    where
        D: SignalGroup + 'static,
        F: Fn(D::Data<'_>) -> Return + 'static,
        Return: WatcherCallbackReturn,
    {
        let mark_cancel = Cell::new(false);
        let strong_listener = Listener::new(|listener| {
            deps.dependent_many(listener);
            let callback_cell = RefCell::new(Some((callback, deps)));

            move |action| {
                if !action.is_update() {
                    return true;
                }

                let keep_alive = if let Some((callback, deps)) = &*callback_cell.borrow() {
                    callback(D::deref_wrapper(&deps.read_many())).keep_alive()
                } else {
                    return false;
                };

                if mark_cancel.get() || !keep_alive {
                    match callback_cell.try_borrow_mut() {
                        Ok(mut cb) => *cb = None,
                        Err(_) => mark_cancel.set(true),
                    };
                    false
                } else {
                    true
                }
            }
        });

        self.push(Watcher(strong_listener));
        self
    }

    fn watch_borrow_mut<T, F, Return, D>(
        &mut self,
        cell: &Rc<RefCell<T>>,
        callback: F,
        deps: D,
    ) -> &mut Self
    where
        T: 'static,
        D: SignalGroup + 'static,
        F: Fn(&mut T, D::Data<'_>) -> Return + 'static,
        Return: WatcherCallbackReturn,
    {
        let cell = cell.clone();
        self.watch(
            move |data| {
                let Ok(mut borrowed) = cell.try_borrow_mut() else {
                    panic!(
                        "cannot borrow `RefCell` as mutable when the watcher triggered. \
                        the callback function is `{}`",
                        type_name::<F>()
                    );
                };
                let ret = callback(&mut borrowed, data).keep_alive();
                ret
            },
            deps,
        );
        self
    }

    fn watch_once<F, Return, D>(&mut self, callback: F, deps: D) -> &mut Self
    where
        D: SignalGroup + 'static,
        F: FnOnce(D::Data<'_>) -> Return + 'static,
    {
        let strong_listener = Listener::new(|listener| {
            deps.dependent_many(listener);
            let callback_cell = Cell::new(Some((callback, deps)));

            move |action| {
                if !action.is_update() {
                    return true;
                }

                if let Some((callback, deps)) = callback_cell.take() {
                    callback(D::deref_wrapper(&deps.read_many()));
                };

                false
            }
        });

        self.push(Watcher(strong_listener));
        self
    }
}

pub trait WatcherCallbackReturn: 'static {
    fn keep_alive(self) -> bool;
}

impl WatcherCallbackReturn for bool {
    fn keep_alive(self) -> bool {
        self
    }
}

impl WatcherCallbackReturn for () {
    fn keep_alive(self) -> bool {
        true
    }
}
