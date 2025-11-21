use std::{
    any::{Any, type_name},
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::hook::{listener::ListenerCallback, utils::CallbackAction};

use super::{Listener, signal_group::SignalGroup};

pub struct Watcher(#[allow(unused)] Rc<dyn Any>);

impl Watcher {
    pub fn watch<F, Return, D>(deps: D, callback: F) -> Self
    where
        D: SignalGroup + 'static,
        F: Fn(D::Data<'_>) -> Return + 'static,
        Return: WatcherCallbackReturn,
    {
        let mark_cancel = Cell::new(false);
        let callback_cell = RefCell::new(Some(callback));

        WatcherListenerCallback {
            deps,
            func: move |action: CallbackAction, deps: &D| {
                if !action.is_update() {
                    return true;
                }

                let keep_alive = if let Some(callback) = &*callback_cell.borrow() {
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
            },
        }
        .make()
    }

    pub fn with<T, F, Return, D>(cell: &Rc<RefCell<T>>, deps: D, callback: F) -> Self
    where
        T: 'static,
        D: SignalGroup + 'static,
        F: Fn(&mut T, D::Data<'_>) -> Return + 'static,
        Return: WatcherCallbackReturn,
    {
        let cell = cell.clone();
        Self::watch(deps, move |data| {
            let Ok(mut borrowed) = cell.try_borrow_mut() else {
                panic!(
                    "cannot borrow `RefCell` as mutable when the watcher triggered. \
                        the callback function is `{}`",
                    type_name::<F>()
                );
            };
            let ret = callback(&mut borrowed, data).keep_alive();
            ret
        })
    }

    pub fn once<F, Return, D>(&mut self, deps: D, callback: F) -> Self
    where
        D: SignalGroup + 'static,
        F: FnOnce(D::Data<'_>) -> Return + 'static,
    {
        let callback_cell = Cell::new(Some(callback));
        WatcherListenerCallback {
            deps,
            func: move |action: CallbackAction, deps: &D| {
                if !action.is_update() {
                    return true;
                }

                if let Some(callback) = callback_cell.take() {
                    callback(D::deref_wrapper(&deps.read_many()));
                };

                false
            },
        }
        .make()
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

struct WatcherListenerCallback<F, D> {
    func: F,
    deps: D,
}

impl<F, D> ListenerCallback for WatcherListenerCallback<F, D>
where
    F: Fn(CallbackAction, &D) -> bool + 'static,
    D: SignalGroup + 'static,
{
    fn call(&self, action: CallbackAction) -> bool {
        (self.func)(action, &self.deps)
    }
}

impl<F, D> WatcherListenerCallback<F, D>
where
    F: Fn(CallbackAction, &D) -> bool + 'static,
    D: SignalGroup + 'static,
{
    fn make(self) -> Watcher {
        let this = Listener::new(self);
        this.callback.deps.dependent_many(this.to_listener());
        Watcher(this)
    }
}
