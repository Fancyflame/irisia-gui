use std::{
    any::Any,
    cell::{Cell, Ref, RefCell},
    rc::Rc,
};

use crate::map_rc::{MapRc, MapWeak};

use super::{
    dep::{DepCollecter, FullVersion, EMPTY_VER},
    DataSource, DepNode, Watchable,
};

pub struct Computed<D> {
    cache: RefCell<Option<D>>,
    slf: MapWeak<Self>,

    binding: MapWeak<dyn Any>,
    compute: &'static dyn Any,
    compute_use: fn(&'static dyn Any, &MapWeak<dyn Any>, cache: Option<D>) -> D,

    last_ver: Cell<FullVersion>,
    marked_dirty: Cell<bool>,
    pub(super) dependents: DepCollecter,
}

impl<D> Computed<D>
where
    D: 'static,
{
    pub fn new<S: 'static>(
        binding: &MapRc<S>,
        compute: &'static fn(&S, Option<D>) -> D,
    ) -> MapRc<Self> {
        fn compute_use<S: 'static, D: 'static>(
            func: &'static dyn Any,
            binding: &MapWeak<dyn Any>,
            cache: Option<D>,
        ) -> D {
            let func = func.downcast_ref::<fn(&S, Option<D>) -> D>().unwrap();
            let binding = MapRc::map(&binding.upgrade().unwrap(), |any| {
                any.downcast_ref::<S>().unwrap()
            });
            func(&binding, cache)
        }

        Rc::new_cyclic(|weak| Computed {
            cache: RefCell::new(None),
            slf: weak.clone().into(),

            binding: MapRc::downgrade(&MapRc::map_to_any(binding)),
            compute: compute as _,
            compute_use: compute_use::<S, D>,

            last_ver: EMPTY_VER.into(),
            marked_dirty: true.into(),
            dependents: DepCollecter::new(),
        })
        .into()
    }
}

impl<D> Watchable for Computed<D>
where
    D: 'static,
{
    type Data = D;

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
            *borrowed = Some((self.compute_use)(
                self.compute,
                &self.binding,
                borrowed.take(),
            ));
            self.dependents.pop_dependent();

            drop(borrowed);
        }

        Ref::map(self.cache.borrow(), |x| x.as_ref().unwrap()).into()
    }

    fn subscribe(&self, sub: &MapRc<dyn DepNode>) {
        self.dependents.subscribe(sub);
    }
}

impl<D> DepNode for Computed<D>
where
    D: 'static,
{
    fn on_update(&self) {
        let cv = self.dependents.current_ver().unwrap();
        if self.last_ver.get() != cv {
            self.last_ver.set(cv);
            self.marked_dirty.set(true);
            self.dependents.update_all();
        }
    }
}
