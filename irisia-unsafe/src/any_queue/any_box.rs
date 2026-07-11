use std::{
    alloc::{self, Layout},
    any::Any,
    ptr::{self, NonNull},
};

use super::type_eliminate::AnyMetadata;

/// A reusable, type-erased owning allocation.
pub struct AnyBox {
    ptr: NonNull<u8>,
    allocation_size: usize,
    allocation_align: usize,
    metadata: Option<&'static dyn AnyMetadata>,
}

impl AnyBox {
    /// Creates an empty box without allocating.
    pub fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            allocation_size: 0,
            allocation_align: 1,
            metadata: None,
        }
    }

    /// Returns whether this box currently owns a value.
    pub fn is_empty(&self) -> bool {
        self.metadata.is_none()
    }

    /// Borrows the current value as a dynamic `Any` reference.
    pub fn as_any(&self) -> Option<&dyn Any> {
        self.metadata
            .map(|metadata| unsafe { &*metadata.restore_pointer(self.ptr.as_ptr().cast()) })
    }

    /// Mutably borrows the current value as a dynamic `Any` reference.
    pub fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        self.metadata
            .map(|metadata| unsafe { &mut *metadata.restore_pointer(self.ptr.as_ptr().cast()) })
    }

    /// Attempts to borrow the current value as `T`.
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.as_any()?.downcast_ref::<T>()
    }

    /// Attempts to mutably borrow the current value as `T`.
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.as_any_mut()?.downcast_mut::<T>()
    }

    pub(crate) fn replace_from_raw(
        &mut self,
        metadata: &'static dyn AnyMetadata,
        source: *const u8,
    ) {
        self.clear_value();

        let layout = metadata.layout();
        self.ensure_allocation(layout);

        if layout.size() != 0 {
            unsafe {
                ptr::copy_nonoverlapping(source, self.ptr.as_ptr(), layout.size());
            }
        }

        self.metadata = Some(metadata);
    }

    fn clear_value(&mut self) {
        if let Some(metadata) = self.metadata.take() {
            unsafe {
                ptr::drop_in_place(metadata.restore_pointer(self.ptr.as_ptr().cast()));
            }
        }
    }

    fn ensure_allocation(&mut self, layout: Layout) {
        let required_size = layout.size().max(1);

        if self.allocation_size >= required_size && self.allocation_align >= layout.align() {
            return;
        }

        if self.allocation_size != 0 && self.allocation_align == layout.align() {
            let old_layout = Layout::from_size_align(self.allocation_size, self.allocation_align)
                .expect("invalid AnyBox allocation layout");
            let new_ptr = unsafe { alloc::realloc(self.ptr.as_ptr(), old_layout, required_size) };
            let new_layout = Layout::from_size_align(required_size, layout.align())
                .expect("invalid AnyBox layout");
            self.ptr =
                NonNull::new(new_ptr).unwrap_or_else(|| alloc::handle_alloc_error(new_layout));
            self.allocation_size = required_size;
            return;
        }

        let new_layout = Layout::from_size_align(required_size, layout.align())
            .expect("invalid AnyBox allocation layout");
        let new_ptr = unsafe { alloc::alloc(new_layout) };
        let new_ptr =
            NonNull::new(new_ptr).unwrap_or_else(|| alloc::handle_alloc_error(new_layout));

        if self.allocation_size != 0 {
            unsafe {
                alloc::dealloc(
                    self.ptr.as_ptr(),
                    Layout::from_size_align_unchecked(self.allocation_size, self.allocation_align),
                );
            }
        }

        self.ptr = new_ptr;
        self.allocation_size = required_size;
        self.allocation_align = layout.align();
    }
}

impl Default for AnyBox {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AnyBox {
    fn drop(&mut self) {
        self.clear_value();

        if self.allocation_size != 0 {
            unsafe {
                alloc::dealloc(
                    self.ptr.as_ptr(),
                    Layout::from_size_align_unchecked(self.allocation_size, self.allocation_align),
                );
            }
        }
    }
}
