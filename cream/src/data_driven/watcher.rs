use crate::map_rc::MapRc;

use super::{DepNode, Watchable};

pub struct Watcher<F, D> {
    watch: MapRc<dyn Watchable<Data = D>>,
    call: F,
}

impl<F, D> Watcher<F, D>
where
    F: Fn(&D) + 'static,
    D: 'static,
{
    pub fn new<W>(watch: &MapRc<W>, call: F) -> MapRc<Self>
    where
        W: Watchable<Data = D> + 'static,
    {
        let watch = MapRc::map(watch, |w| w as &dyn Watchable<Data = D>);
        let wacther = MapRc::new(Watcher {
            watch: watch.clone(),
            call,
        });

        watch.subscribe(&MapRc::map(&wacther, |w| w as _));
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
