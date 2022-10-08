use crate::map_rc::MapRc;

use super::{DepNode, Watchable};

pub struct Watcher<F, D> {
    watch: MapRc<dyn Watchable<D>>,
    call: F,
}

impl<F, D> Watcher<F, D>
where
    F: Fn(&D) + 'static,
    D: 'static,
{
    pub fn new<W>(watch: &MapRc<W>, call: F) -> MapRc<Self>
    where
        W: Watchable<D> + 'static,
    {
        let watch = watch.map(|w| w as &dyn Watchable<D>);
        let wacther = MapRc::new(Watcher {
            watch: watch.clone(),
            call,
        });

        watch.subscribe(&wacther.map(|w| w as _));
        wacther
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
