use std::{
    alloc::{dealloc, Layout},
    mem::ManuallyDrop,
};

use thiserror::Error;

#[derive(Debug)]
pub struct ReuseVec {
    ptr: *mut (),
    layout: Layout,
    capacity: usize,
}

impl<T> From<Vec<T>> for ReuseVec {
    fn from(value: Vec<T>) -> Self {
        let mut vec = ManuallyDrop::new(value);
        ReuseVec {
            ptr: vec.as_mut_ptr().cast(),
            layout: Layout::new::<T>(),
            capacity: vec.capacity(),
        }
    }
}

impl ReuseVec {
    pub fn try_into_vec<T>(self) -> Result<Vec<T>, ReuseVecLayoutError> {
        let this_layout = Layout::new::<T>();
        if self.layout != this_layout {
            return Err(ReuseVecLayoutError {
                expected: self.layout,
                provided: this_layout,
                reuse_vec: self,
            });
        }

        let this = ManuallyDrop::new(self);
        unsafe { Ok(Vec::from_raw_parts(this.ptr.cast(), 0, this.capacity)) }
    }
}

impl<T> From<ReuseVec> for Vec<T> {
    fn from(this: ReuseVec) -> Self {
        this.try_into_vec().unwrap_or_else(|_| Vec::new())
    }
}

impl Drop for ReuseVec {
    fn drop(&mut self) {
        let size = self.layout.size() * self.capacity;

        if size != 0 {
            let layout = Layout::from_size_align(size, self.layout.align()).unwrap();
            unsafe {
                dealloc(self.ptr.cast(), layout);
            }
        }
    }
}

#[derive(Debug, Error)]
#[error(
    "cannot convert `ReuseVec` to `Vec`, because the memory layout doesn't match. \
    expected: {expected:?}, \
    provided: {provided:?}"
)]
pub struct ReuseVecLayoutError {
    pub reuse_vec: ReuseVec,
    pub expected: Layout,
    pub provided: Layout,
}
