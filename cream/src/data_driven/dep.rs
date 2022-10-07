use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::{Rc, Weak},
};

use super::{data::Version, thread_guard::ThreadGuard, Data};

lazy_static! {
    static ref COLLECTER_STACK: ThreadGuard<Static> = ThreadGuard::new(Static {
        collecter_stack: RefCell::new(Vec::new()),
        update_ver: Cell::new(None)
    });
}

pub type FullVersion = (*const (), Version);
pub const EMPTY_VER: FullVersion = (std::ptr::null(), false);

pub struct Static {
    collecter_stack: RefCell<Vec<Rc<dyn DepNode>>>,
    update_ver: Cell<Option<FullVersion>>, // Some: is updating; None: not updating.
}

pub struct DepCollecter {
    watchers: RefCell<HashMap<*const dyn DepNode, Weak<dyn DepNode>>>,
    stc: &'static Static,
}

impl DepCollecter {
    pub fn new() -> Self {
        DepCollecter {
            watchers: RefCell::new(HashMap::new()),
            stc: &*COLLECTER_STACK,
        }
    }

    #[inline]
    pub fn current_ver(&self) -> Option<FullVersion> {
        self.stc.update_ver.get()
    }

    #[inline]
    pub fn push_dependent<T: DepNode + 'static>(&self, node: &Rc<T>) {
        self.stc.collecter_stack.borrow_mut().push(node.clone());
    }

    #[inline]
    pub fn pop_dependent(&self) {
        self.stc.collecter_stack.borrow_mut().pop();
    }

    pub fn collect(&self) {
        if let Some(watcher) = self.stc.collecter_stack.borrow().last() {
            self.subscribe(watcher);
        }
    }

    pub fn update_root<D>(&self, data: &Rc<Data<D>>) {
        self.stc
            .update_ver
            .set(Some((Rc::as_ptr(data) as _, data.version())));
        self.update_all();
        self.stc.update_ver.set(None);
    }

    pub fn update_all(&self) {
        self.watchers
            .borrow_mut()
            .retain(|_, dep| match Weak::upgrade(dep) {
                Some(dep) => {
                    dep.on_update();
                    true
                }
                None => false,
            })
    }

    #[inline]
    pub fn subscribe(&self, sub: &Rc<dyn DepNode>) {
        self.watchers
            .borrow_mut()
            .insert(Rc::as_ptr(sub), Rc::downgrade(sub));
    }
}

/*impl Drop for DepCollecter {
    fn drop(&mut self) {
        for (_, dep) in self.watchers.get_mut().iter() {
            if let Some(rc) = Weak::upgrade(dep) {
                rc.on_dropped();
            }
        }
    }
}*/

/// Implement this trait says that it can be a dependent of a
/// data source.
pub trait DepNode {
    fn on_update(&self) {}
    //fn on_dropped(&self) {}
}
