use std::{any::Any, rc::Weak};

use stack_box::{coerce, FitStackBox};

type InnerBox<T> = FitStackBox!(T, Inner<dyn Any, fn()>);

pub struct Listener(InnerBox<dyn Callback>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CallbackAction {
    Update,
    RegisterDirty,
    ClearDirty,
}

impl Listener {
    /// a compile error if callback is larger than a function pointer
    pub(crate) fn new<T, F>(src: Weak<T>, callback: F) -> Self
    where
        T: ?Sized + 'static,
        F: Fn(&T, CallbackAction) + Copy + 'static,
    {
        let ib = InnerBox::new(Inner { src, callback });
        Listener(coerce!(ib))
    }

    pub(crate) fn callback(&self, action: CallbackAction) -> bool {
        self.0.callback(action)
    }
}

struct Inner<T: ?Sized, F> {
    src: Weak<T>,
    callback: F,
}

trait Callback {
    fn callback(&self, action: CallbackAction) -> bool;
    fn unsize_clone(&self) -> InnerBox<dyn Callback>;
}

impl<T, F> Callback for Inner<T, F>
where
    T: ?Sized + 'static,
    F: Fn(&T, CallbackAction) + Copy + 'static,
{
    fn callback(&self, action: CallbackAction) -> bool {
        match self.src.upgrade() {
            Some(rc) => {
                (self.callback)(&rc, action);
                true
            }
            None => false,
        }
    }

    fn unsize_clone(&self) -> InnerBox<dyn Callback> {
        coerce!(InnerBox::new(Inner {
            src: self.src.clone(),
            callback: self.callback,
        }))
    }
}

impl Clone for Listener {
    fn clone(&self) -> Self {
        Self(self.0.unsize_clone())
    }
}
