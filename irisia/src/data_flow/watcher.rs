use std::{
    cell::{Cell, RefCell, RefMut},
    rc::{Rc, Weak},
};

use super::{deps::Listener, Listenable, Wakeable};

pub struct Watcher {
    inner: Rc<Inner<dyn OptionedFn>>,
}

struct Inner<Of: ?Sized> {
    delay_set_unwatch: Cell<bool>,
    type_name: &'static str,
    this: Weak<dyn Wakeable>,
    callback: RefCell<Of>,
}

impl Watcher {
    pub fn new<F>(callback: F) -> Self
    where
        F: FnMut() -> bool,
    {
        let inner = Rc::new_cyclic(|this| Inner {
            type_name: callback.fn_type_name(),
            delay_set_unwatch: Cell::new(false),
            this: this.clone(),
            callback: RefCell::new(Some(callback)),
        });

        Self { inner }
    }

    fn get_optioned_fn(&self) -> RefMut<dyn OptionedFn> {
        match self.inner.callback.try_borrow_mut() {
            Ok(cb) => cb,
            Err(_) => {
                panic!(
                    "cyclic callback triggered, please do not mutate values \
                    that this watcher may directly or indirectly watching. \
                    type of the callback function is `{}`",
                    self.type_name
                );
            }
        }
    }

    pub fn call(&self) -> bool {
        let mut optioned_fn = self.get_optioned_fn();
        match &mut *optioned_fn {
            Some(f) => {
                let keep_watching = f() && !self.delay_set_unwatch.get();
                if !keep_watching {
                    *optioned_fn = None;
                }
                keep_watching
            }
            None => false,
        }
    }

    pub fn destroy(&self) {
        match self.callback.try_borrow_mut() {
            Ok(mut cb) => *cb = None,
            Err(_) => {
                self.delay_set_unwatch.set(true);
            }
        }
    }

    pub fn watch<T: Listenable>(&self, target: &T) -> &Self {
        let listener = Listener::Rc(self.this.upgrade().unwrap());
        target.add_listener(&listener);
        self
    }

    pub fn unwatch<T: Listenable>(&self, target: &T) -> &Self {
        let listener = Listener::Rc(self.this.upgrade().unwrap());
        target.remove_listener(&listener);
        self
    }
}

impl Wakeable for Watcher {
    fn add_back_reference(&self, _: &Rc<dyn super::Listenable>) {}
    fn set_dirty(&self) {}
    fn wake(&self) -> bool {
        self.get_optioned_fn().call()
    }
}

trait OptionedFn {
    fn drop(&mut self);
    fn call(&mut self) -> bool;
}

impl<F> OptionedFn for Option<F>
where
    F: FnMut() -> bool,
{
    fn drop(&mut self) {
        *self = None
    }

    fn call(&mut self) -> bool {
        match self {
            Some(f) => {
                let keep_watching = f();
                if !keep_watching {
                    *self = None;
                }
                keep_watching
            }
            None => false,
        }
    }
}
