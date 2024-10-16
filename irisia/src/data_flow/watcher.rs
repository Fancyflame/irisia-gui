use std::{
    cell::{Cell, RefCell, RefMut},
    rc::{Rc, Weak},
};

use super::{deps::Listener, ToListener, Wakeable};

#[derive(Clone)]
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
        F: FnMut() -> bool + 'static,
    {
        let inner = Rc::new_cyclic(|this: &Weak<Inner<_>>| Inner {
            type_name: std::any::type_name::<F>(),
            delay_set_unwatch: Cell::new(false),
            this: this.clone(),
            callback: RefCell::new(Some(callback)),
        });

        Self { inner }
    }

    pub fn call(&self) -> bool {
        self.inner.get_optioned_fn().call()
    }

    pub fn drop_callback(&self) {
        match self.inner.callback.try_borrow_mut() {
            Ok(mut refm) => refm.drop(),
            Err(_) => self.inner.delay_set_unwatch.set(true),
        }
    }
}

impl<Of> Inner<Of>
where
    Of: OptionedFn + ?Sized,
{
    fn get_optioned_fn(&self) -> RefMut<Of> {
        match self.callback.try_borrow_mut() {
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
}

impl<Of> Wakeable for Inner<Of>
where
    Of: OptionedFn + ?Sized,
{
    fn add_back_reference(&self, _: &Rc<dyn super::Listenable>) {}
    fn set_dirty(&self) {}
    fn wake(&self) -> bool {
        let mut refm = self.get_optioned_fn();
        let keep_watching = refm.call() && !self.delay_set_unwatch.get();
        if !keep_watching {
            refm.drop();
        }
        keep_watching
    }
}

trait OptionedFn {
    fn drop(&mut self);
    fn call(&mut self) -> bool;
}

impl<F> OptionedFn for Option<F>
where
    F: FnMut() -> bool + 'static,
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

impl ToListener for Watcher {
    fn to_listener(&self) -> Listener {
        Listener::Rc(self.inner.this.upgrade().unwrap())
    }
}
