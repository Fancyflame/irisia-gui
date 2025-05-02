use std::cell::{Cell, RefCell};

use smallvec::SmallVec;

use super::{listener::StrongListener, signal_group::SignalGroup, Listener};

pub struct WatcherList(SmallVec<[Watcher; 1]>);

impl WatcherGuard for WatcherList {
    fn push(&mut self, watcher: Watcher) {
        self.0.push(watcher);
    }
}

pub struct Watcher(StrongListener);

pub trait WatcherGuard {
    fn push(&mut self, watcher: Watcher);

    // Provided

    fn watch<F, Return, D>(&mut self, callback: F, deps: D)
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
    }

    fn watch_once<F, Return, D>(&mut self, callback: F, deps: D)
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
