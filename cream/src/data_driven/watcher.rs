use std::rc::{Rc, Weak};

use super::{DepNode, Watchable};

pub struct Watcher<D: 'static> {
    watch: Rc<dyn Watchable<Data = D>>,
    call: Box<dyn Fn(&D)>,
}

impl<D> Watcher<D> {
    pub fn new<W, S, F>(watch: &Rc<W>, binding: Weak<S>, call: F) -> Rc<Self>
    where
        S: 'static,
        W: Watchable<Data = D> + 'static,
        F: Fn(&S, &D) + 'static,
    {
        let wather = Rc::new(Watcher {
            watch: watch.clone(),
            call: Box::new(move |data| {
                let rc = binding.upgrade().expect("The binding has been dropped");
                call(&rc, data);
            }),
        });

        watch.subscribe(&(wather.clone() as _));
        wather
    }
}

impl<D> DepNode for Watcher<D> {
    fn on_update(&self) {
        (self.call)(&*self.watch.get());
    }
}
