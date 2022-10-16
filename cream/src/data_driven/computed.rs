use std::{
    cell::{Ref, RefCell},
    rc::{Rc, Weak},
};

use super::{
    dep::{DepCollecter, FullVersion, EMPTY_VER},
    DataSource, DepNode, Watchable,
};

pub struct Computed<D> {
    ver: RefCell<ComputedVer<D>>,
    slf: Weak<Self>,
    compute: Box<dyn Fn(Option<D>) -> D>,
    pub(super) dependents: DepCollecter,
}

struct ComputedVer<D> {
    cache: Option<D>,
    last_ver: FullVersion,
    marked_dirty: bool,
}

impl<D> Computed<D>
where
    D: 'static,
{
    pub fn new<S: 'static>(binding: Weak<S>, compute: fn(&S, Option<D>) -> D) -> Rc<Self> {
        Rc::new_cyclic(|weak| Computed {
            ver: RefCell::new(ComputedVer {
                cache: None,
                last_ver: EMPTY_VER.into(),
                marked_dirty: true.into(),
            }),
            slf: weak.clone(),
            compute: Box::new(move |opt| {
                let rc = binding.upgrade().expect("The binding has been dropped");
                compute(&rc, opt)
            }),
            dependents: DepCollecter::new(),
        })
    }
}

impl<D> Watchable for Computed<D>
where
    D: 'static,
{
    type Data = D;

    fn get(&self) -> DataSource<D> {
        self.dependents.collect();

        let mut cache = self.ver.borrow_mut();
        let is_dirty = match (cache.marked_dirty, self.dependents.current_ver()) {
            (true, _) => true,
            (false, Some(ver)) => ver != cache.last_ver,
            (false, None) => false,
        };

        if is_dirty {
            cache.marked_dirty = false;
            if let Some(cv) = self.dependents.current_ver() {
                cache.last_ver = cv;
            }

            self.dependents.push_dependent(&self.slf.upgrade().unwrap());
            cache.cache = Some((self.compute)(cache.cache.take()));
            self.dependents.pop_dependent();
        }

        drop(cache);
        Ref::map(self.ver.borrow(), |x| x.cache.as_ref().unwrap()).into()
    }

    fn subscribe(&self, sub: &Rc<dyn DepNode>) {
        self.dependents.subscribe(sub);
    }
}

impl<D> DepNode for Computed<D>
where
    D: 'static,
{
    fn on_update(&self) {
        let mut cache = self.ver.borrow_mut();
        let cv = self.dependents.current_ver().unwrap();
        if cache.last_ver != cv {
            cache.last_ver = cv;
            cache.marked_dirty = true;
            drop(cache);
            self.dependents.update_all();
        }
    }
}
