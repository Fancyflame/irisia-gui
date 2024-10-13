use std::rc::{Rc, Weak};

use super::{deps::DepdencyList, Listener, Wakeable};

pub struct Observer {
    inner: Rc<Inner<dyn Fn() -> bool>>,
}

struct Inner<F: ?Sized> {
    deps: DepdencyList,
    trigger_fn: F,
}

impl Observer {
    pub fn new<F>(trigger_fn: F) -> Self
    where
        F: Fn() -> bool + 'static,
    {
        Observer {
            inner: Rc::new_cyclic(|this: &Weak<Inner<_>>| Inner {
                trigger_fn,
                deps: DepdencyList::new(Listener::Weak(this.clone())),
            }),
        }
    }

    pub fn invoke<F2, R>(&self, f: F2) -> R
    where
        F2: FnOnce() -> R,
    {
        self.inner.deps.collect_dependencies(f)
    }
}

impl<F> Wakeable for Inner<F>
where
    F: Fn() -> bool + ?Sized,
{
    fn add_back_reference(&self, dep: &Rc<dyn super::Listenable>) {
        self.deps.add_dependency(dep);
    }

    fn set_dirty(&self) {}

    fn wake(&self) -> bool {
        let keep_watching = (self.trigger_fn)();
        if !keep_watching {
            self.deps.clear();
        }
        true
    }
}
