use std::{any::Any, rc::Weak};

use stack_box::{coerce, FitStackBox};

use super::ResInner;

type InnerBox<T> = FitStackBox!(T, Inner<dyn Any>);

pub struct Listener(InnerBox<dyn Callback>);

impl Listener {
    pub(super) fn new<T>(
        src: Weak<ResInner<T>>,
        callback: fn(&mut T),
        dirty_setter: fn(&mut T),
    ) -> Self
    where
        T: ?Sized + 'static,
    {
        let ib = InnerBox::new(Inner {
            src,
            callback,
            dirty_setter,
        });
        Listener(coerce!(ib))
    }

    pub(crate) fn set_dirty(&self) -> bool {
        self.0.set_dirty()
    }

    pub(crate) fn call(&self) -> bool {
        self.0.call()
    }
}

struct Inner<T: ?Sized> {
    src: Weak<ResInner<T>>,
    callback: fn(&mut T),
    dirty_setter: fn(&mut T),
}

impl<T: ?Sized> Inner<T> {
    fn src_mut<F>(&self, f: F) -> bool
    where
        F: FnOnce(&mut T),
    {
        let Some(rc) = self.src.upgrade() else {
            return false;
        };

        let retain = match rc.value.try_borrow_mut() {
            Ok(mut src) => {
                f(&mut src);
                true
            }
            Err(_) => {
                if cfg!(debug_assertions) {
                    panic!("cannot trigger the callback inside the callback itself");
                } else {
                    false
                }
            }
        };

        retain
    }
}

trait Callback {
    fn set_dirty(&self) -> bool;
    fn call(&self) -> bool;
}

impl<T: ?Sized> Callback for Inner<T> {
    fn set_dirty(&self) -> bool {
        self.src_mut(self.dirty_setter)
    }

    fn call(&self) -> bool {
        self.src_mut(self.callback)
    }
}
