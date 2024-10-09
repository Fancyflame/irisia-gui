use std::{cell::Cell, ops::Deref, rc::Rc};

use super::{deps::Listener, Listenable, Wakeable};

#[derive(Clone)]
pub struct DirtyFlag {
    inner: Rc<Inner>,
}

struct Inner {
    flag: Cell<bool>,
    this: Listener,
}

impl DirtyFlag {
    pub fn new() -> Self {
        Self(Rc::new_cyclic(|this| Inner {
            flag: Cell::new(false),
            this: Listener::Weak(this.clone()),
        }))
    }

    pub const fn is_dirty(&self) -> bool {
        self.inner.flag.get()
    }

    pub fn set_clean(&self) {
        self.inner.flag.set(false);
    }

    pub fn listener(&self) -> &Listener {
        &self.inner.this
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
    type Target = Rc<dyn Wakeable>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
