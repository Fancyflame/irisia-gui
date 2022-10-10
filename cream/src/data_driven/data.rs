use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::map_rc::{MapRc, MapWeak};

use super::{dep::DepCollecter, DataSource, DepNode, Watchable};

pub(super) type Version = bool;

pub struct Data<D> {
    data: RefCell<D>,
    slf: MapWeak<Self>,
    version: Cell<Version>,
    dependents: DepCollecter,
}

impl<D: 'static> Data<D> {
    pub fn new(default: D) -> MapRc<Self> {
        Rc::new_cyclic(|weak| Data {
            data: RefCell::new(default),
            slf: weak.clone().into(),
            version: Cell::new(false),
            dependents: DepCollecter::new(),
        })
        .into()
    }

    #[inline]
    pub fn set<'a>(&self, val: D) {
        *self.data.borrow_mut() = val;
        self.dependents.update_root(&self.slf.upgrade().unwrap());
    }

    #[inline]
    pub(super) fn version(&self) -> Version {
        self.version.get()
    }
}

impl<D> Watchable for Data<D> {
    type Data = D;

    fn get<'a>(&'a self) -> DataSource<D> {
        self.dependents.collect();
        self.data.borrow().into()
    }

    fn subscribe(&self, sub: &MapRc<dyn DepNode>) {
        self.dependents.subscribe(sub);
    }
}

/*pub struct RefMut<'a, D> {
    data: &'a Rc<Data<D>>,
    ref_mut: Option<RefCellRefMut<'a, D>>,
}

impl<D> Deref for RefMut<'_, D> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        /*#[cfg(debug_assertions)]
        {
            if let Some(waker) = get_dependent(&self.data.stack) {
                if !self.data.dependents.borrow().contains(&waker) {
                    println!("DEBUG WARNING: Reading value through `get_mut` is not responding, TRY USING `get_mut_read` \
                        INSTEAD. This information is not always displayed when incorrect reading happens, such as reading \
                        from `&mut T` rather than `Deref` or under release mode. So, it's suggested to review your code \
                        to see if there is incorrect usage.
                    ");
                }
            }
        }*/
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
        self.ref_mut = None;
        self.data.update();
    }
}
*/
