use std::{
    cell::{Cell, RefCell, RefMut as RefCellRefMut},
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

use super::{dep::DepCollecter, DataSource, DepNode, Watchable};

pub(super) type Version = bool;

pub struct MutData<D> {
    data: RefCell<D>,
    slf: Weak<Self>,
    version: Cell<Version>,
    dependents: DepCollecter,
}

impl<D: 'static> MutData<D> {
    pub fn new(default: D) -> Rc<Self> {
        Rc::new_cyclic(|slf| MutData {
            data: RefCell::new(default),
            slf: slf.clone().into(),
            version: Cell::new(false),
            dependents: DepCollecter::new(),
        })
    }

    fn update(&self) {
        self.dependents.update_root(&self.slf.upgrade().unwrap());
    }

    #[inline]
    pub fn set<'a>(&self, val: D) {
        *self.data.borrow_mut() = val;
        self.update();
    }

    pub fn borrow_mut(&self) -> RefMut<D> {
        RefMut {
            data: &self,
            ref_mut: Some(self.data.borrow_mut()),
        }
    }

    #[inline]
    pub(super) fn version(&self) -> Version {
        self.version.get()
    }
}

impl<D> Watchable for MutData<D> {
    type Data = D;

    fn get<'a>(&'a self) -> DataSource<D> {
        self.dependents.collect();
        self.data.borrow().into()
    }

    fn subscribe(&self, sub: &Rc<dyn DepNode>) {
        self.dependents.subscribe(sub);
    }
}

pub struct RefMut<'a, D: 'static> {
    data: &'a MutData<D>,
    ref_mut: Option<RefCellRefMut<'a, D>>,
}

impl<D> Deref for RefMut<'_, D> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        self.ref_mut.as_ref().unwrap()
    }
}

impl<D> DerefMut for RefMut<'_, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.ref_mut.as_mut().unwrap()
    }
}

impl<D> Drop for RefMut<'_, D> {
    fn drop(&mut self) {
        drop(self.ref_mut.take());
        self.data.update();
    }
}
