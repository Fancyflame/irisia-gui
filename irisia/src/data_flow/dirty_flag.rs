use std::{cell::Cell, ops::Deref, rc::Rc};

use super::{deps::Listener, Listenable, ToListener, Wakeable};

#[derive(Clone)]
pub struct DirtyFlag {
    inner: Rc<Inner>,
}

struct Inner {
    flag: Cell<bool>,
}

impl DirtyFlag {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(Inner {
                flag: Cell::new(false),
            }),
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.inner.flag.get()
    }

    pub fn set_clean(&self) {
        self.inner.flag.set(false);
    }
}

impl Wakeable for Inner {
    fn add_back_reference(&self, _: &Rc<dyn Listenable>) {}

    fn set_dirty(&self) {
        self.flag.set(true);
    }

    fn wake(&self) -> bool {
        true
    }
}

impl Deref for DirtyFlag {
    type Target = dyn Wakeable;
    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl ToListener for DirtyFlag {
    fn to_listener(&self) -> Listener {
        Listener::Weak(Rc::downgrade(&self.inner) as _)
    }
}
