use std::rc::{Rc, Weak};

use super::{listener_list::ListenerList, Listener, Wakeable};

pub type RcObserver = Rc<Observer<dyn Fn()>>;

pub struct Observer<F: ?Sized> {
    this: Weak<dyn Wakeable>,
    trigger_fn: F,
}

impl<F> Observer<F>
where
    F: Fn() + ?Sized + 'static,
{
    pub fn new(trigger_fn: F) -> RcObserver
    where
        F: Sized,
    {
        Rc::new_cyclic(|this| Self {
            trigger_fn,
            this: this.clone() as _,
        })
    }

    pub fn invoke<F2, R>(self: &Rc<Self>, f: F2) -> R
    where
        F2: FnOnce() -> R,
    {
        // `F` is unsized, so `Rc<Self>` couldn't be downgraded
        ListenerList::push_global_stack(Listener::Weak(self.this.clone()));
        let r = f();
        ListenerList::pop_global_stack();
        r
    }
}

impl<F> Wakeable for Observer<F>
where
    F: Fn() + ?Sized,
{
    fn update(self: Rc<Self>) -> bool {
        (self.trigger_fn)();
        false
    }
}
