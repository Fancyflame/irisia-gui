use std::{
    any::Any,
    ops::Deref,
    rc::{Rc, Weak},
};

trait Nothing {}
impl<T> Nothing for T {}

pub struct MapRc<T: ?Sized> {
    dst: *const T,
    rc: Rc<dyn Nothing>,
}

impl<T: ?Sized> Clone for MapRc<T> {
    fn clone(&self) -> Self {
        MapRc {
            dst: self.dst,
            rc: self.rc.clone(),
        }
    }
}

impl<T: 'static> MapRc<T> {
    pub fn new(value: T) -> Self {
        Rc::new(value).into()
    }

    #[inline]
    pub fn map_to_any(&self) -> MapRc<dyn Any> {
        self.map(|x| x as _)
    }
}

impl<T: ?Sized> MapRc<T> {
    pub fn map<U, F>(&self, func: F) -> MapRc<U>
    where
        U: ?Sized,
        F: FnOnce(&T) -> &U,
    {
        let ptr = unsafe { func(&*self.dst) as *const U };
        MapRc {
            dst: ptr,
            rc: self.rc.clone(),
        }
    }

    pub fn downgrade(&self) -> MapWeak<T> {
        MapWeak {
            dst: self.dst,
            weak: Rc::downgrade(&self.rc),
        }
    }

    pub fn as_ptr(&self) -> *const T {
        self.dst
    }
}

impl<T: 'static> From<Rc<T>> for MapRc<T> {
    fn from(rc: Rc<T>) -> Self {
        MapRc {
            dst: Rc::as_ptr(&rc),
            rc: rc as _,
        }
    }
}

impl<T: ?Sized> Deref for MapRc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.dst }
    }
}

pub struct MapWeak<T: ?Sized> {
    dst: *const T,
    weak: Weak<dyn Nothing>,
}

impl<T: ?Sized> MapWeak<T> {
    pub fn upgrade(&self) -> Option<MapRc<T>> {
        let mr = MapRc {
            dst: self.dst,
            rc: self.weak.upgrade()?,
        };
        Some(mr)
    }
}

impl<T: 'static> From<Weak<T>> for MapWeak<T> {
    fn from(weak: Weak<T>) -> Self {
        MapWeak {
            dst: weak.as_ptr(),
            weak: weak as _,
        }
    }
}
