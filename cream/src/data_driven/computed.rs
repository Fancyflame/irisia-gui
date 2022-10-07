use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

use super::{
    dep::{DepCollecter, DepNode, FullVersion, EMPTY_VER},
    Watchable,
};

pub struct Computed<F, D> {
    cache: RefCell<Option<D>>,
    slf: Weak<Self>,
    compute: F,
    last_ver: FullVersion,
    marked_dirty: bool,
    pub(super) dependents: DepCollecter,
}

impl<F, D> Computed<F, D>
where
    F: Fn(Option<D>) -> D,
{
    pub fn new(compute: F) -> Rc<Self> {
        Rc::new_cyclic(|slf| Computed {
            cache: RefCell::new(None),
            slf: slf.clone(),
            compute,
            last_ver: EMPTY_VER,
            marked_dirty: true,
            dependents: DepCollecter::new(),
        })
    }
}

impl<F, D: 'static> Watchable<D> for Computed<F, D>
where
    F: Fn(Option<D>) -> D,
{
    fn get(&self) -> Ref<D> {
        self.dependents.collect();
        let is_dirty = match (self.marked_dirty, self.dependents.current_ver()) {
            (true, _) => true,
            (false, Some(ver)) => ver != self.last_ver,
            (false, None) => false,
        };

        if is_dirty {
            self.marked_dirty = false;
            if let Some(cv) = self.dependents.current_ver() {
                self.last_ver = cv;
            }

            let mut borrowed = self.cache.borrow_mut();

            self.dependents
                .push_dependent(&Weak::upgrade(&self.slf).unwrap());
            *borrowed = Some((self.compute)(borrowed.take()));
            self.dependents.pop_dependent();

            drop(borrowed);
        }

        Ref::map(self.cache.borrow(), |x| x.as_ref().unwrap())
    }

    fn subscribe(&self, sub: &Rc<dyn DepNode>) {
        self.dependents.subscribe(sub);
    }
}

impl<F, D> DepNode for Computed<F, D> {
    fn on_update(&self) {
        let cv = self.dependents.current_ver().unwrap();
        if self.last_ver != cv {
            self.last_ver = cv;
            self.marked_dirty = true;
            self.dependents.update_all();
        }
    }
}
