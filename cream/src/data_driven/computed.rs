use std::{
    cell::{Cell, Ref, RefCell},
    rc::Rc,
};

use crate::map_rc::{MapRc, MapWeak};

use super::{
    dep::{DepCollecter, FullVersion, EMPTY_VER},
    DataSource, DepNode, Watchable,
};

pub struct Computed<F, D> {
    cache: RefCell<Option<D>>,
    slf: MapWeak<Self>,
    compute: F,
    last_ver: Cell<FullVersion>,
    marked_dirty: Cell<bool>,
    pub(super) dependents: DepCollecter,
}

impl<F: 'static, D: 'static> Computed<F, D>
where
    F: Fn(Option<D>) -> D,
{
    pub fn new(compute: F) -> MapRc<Self> {
        Rc::new_cyclic(|weak| Computed {
            cache: RefCell::new(None),
            slf: weak.clone().into(),
            compute,
            last_ver: EMPTY_VER.into(),
            marked_dirty: true.into(),
            dependents: DepCollecter::new(),
        })
        .into()
    }
}

impl<F: 'static, D: 'static> Watchable<D> for Computed<F, D>
where
    F: Fn(Option<D>) -> D,
{
    fn get(&self) -> DataSource<D> {
        self.dependents.collect();
        let is_dirty = match (self.marked_dirty.get(), self.dependents.current_ver()) {
            (true, _) => true,
            (false, Some(ver)) => ver != self.last_ver.get(),
            (false, None) => false,
        };

        if is_dirty {
            self.marked_dirty.set(false);
            if let Some(cv) = self.dependents.current_ver() {
                self.last_ver.set(cv);
            }

            let mut borrowed = self.cache.borrow_mut();

            self.dependents.push_dependent(&self.slf.upgrade().unwrap());
            *borrowed = Some((self.compute)(borrowed.take()));
            self.dependents.pop_dependent();

            drop(borrowed);
        }

        Ref::map(self.cache.borrow(), |x| x.as_ref().unwrap()).into()
    }

    fn subscribe(&self, sub: &MapRc<dyn DepNode>) {
        self.dependents.subscribe(sub);
    }
}

impl<F, D> DepNode for Computed<F, D> {
    fn on_update(&self) {
        let cv = self.dependents.current_ver().unwrap();
        if self.last_ver.get() != cv {
            self.last_ver.set(cv);
            self.marked_dirty.set(true);
            self.dependents.update_all();
        }
    }
}
