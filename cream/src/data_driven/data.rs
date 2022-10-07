use std::{
    cell::{Cell, Ref, RefCell},
    rc::Rc,
};

use super::{
    dep::{DepCollecter, DepNode},
    Watchable,
};

pub(super) type Version = bool;

pub struct Data<D> {
    data: RefCell<D>,
    version: Cell<Version>,
    dependents: DepCollecter,
}

impl<D> Data<D> {
    fn new_raw(default: D) -> Self {
        Data {
            data: RefCell::new(default),
            version: Cell::new(false),
            dependents: DepCollecter::new(),
        }
    }

    #[inline]
    pub fn new(default: D) -> Rc<Self> {
        Rc::new(Self::new_raw(default))
    }

    #[inline]
    pub fn set<'a>(self: &Rc<Self>, val: D) {
        *self.data.borrow_mut() = val;
        self.dependents.update_root(self);
    }

    #[inline]
    pub(super) fn version(&self) -> Version {
        self.version.get()
    }
}

impl<D> Watchable<D> for Data<D> {
    fn get<'a>(&'a self) -> Ref<'a, D> {
        self.dependents.collect();
        self.data.borrow()
    }

    fn subscribe(&self, sub: &Rc<dyn DepNode>) {
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
