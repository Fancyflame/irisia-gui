use std::ptr::NonNull;

use self::inner::RcInner;

mod inner;

pub struct Rc<T: ?Sized>(NonNull<RcInner<T>>);
pub struct Weak<T: ?Sized>(NonNull<RcInner<T>>);
pub struct Arc<T: ?Sized>(NonNull<RcInner<T>>);
pub struct AWeak<T: ?Sized>(NonNull<RcInner<T>>);

impl<T: ?Sized> Rc<T> {
    pub fn new(data: T) -> Self
    where
        T: Sized,
    {
        unsafe { Rc(NonNull::new(RcInner::new(data)).unwrap_unchecked()) }
    }

    pub fn downgrade(&self) -> Weak<T> {
        unsafe {
            RcInner::downgrade_rc(self.0.as_ptr());
            Weak(self.0)
        }
    }

    pub fn upgrade_atomic(&self) -> Arc<T> {
        unsafe {
            RcInner::upgrade_rc(self.0.as_ptr());
            Arc(self.0)
        }
    }
}
