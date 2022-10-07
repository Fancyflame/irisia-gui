use std::{cell::Ref, rc::Rc};

use super::dep::DepNode;

pub struct Watcher<F, D> {
    watch: Rc<dyn Watchable<D>>,
    call: F,
}

impl<F, D> Watcher<F, D>
where
    F: Fn(&D) + 'static,
    D: 'static,
{
    pub fn new<W>(watch: Rc<W>, call: F) -> Rc<Self>
    where
        W: Watchable<D> + 'static,
    {
        let w = Rc::new(Watcher { watch, call });

        let w_cloned = w.clone() as Rc<dyn DepNode>;
        w.watch.subscribe(&w_cloned);

        w
    }
}

impl<F, D> DepNode for Watcher<F, D>
where
    F: Fn(&D),
{
    fn on_update(&self) {
        (self.call)(&*self.watch.get());
    }
}

pub trait Watchable<D> {
    fn get<'a>(&'a self) -> Ref<'a, D>;
    fn subscribe(&self, sub: &Rc<dyn DepNode>);
}

/*pub trait WatchShortcut<D: 'static>
where
    Self: Sized,
    for<'r> &'r Self: Into<Watchable<D>>,
{
    #[inline]
    fn watch<F>(&self, call: F) -> Rc<Watcher<D>>
    where
        F: FnMut(&D, &Watcher<D>) + 'static,
    {
        Watcher::new(self, call)
    }
}

impl<T, D: 'static> WatchShortcut<D> for T where for<'r> &'r T: Into<Watchable<D>> + Clone {}*/
