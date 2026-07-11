use std::{alloc::Layout, any::Any, marker::PhantomData};

pub unsafe trait AnyMetadata {
    fn layout(&self) -> Layout;
    fn restore_pointer(&self, addr: *mut ()) -> *mut dyn Any;
}

struct MetadataType<T>(PhantomData<T>);

unsafe impl<T: 'static> AnyMetadata for MetadataType<T> {
    fn layout(&self) -> Layout {
        Layout::new::<T>()
    }
    fn restore_pointer(&self, addr: *mut ()) -> *mut dyn Any {
        addr.cast::<T>()
    }
}

pub fn get_any_metadata<T: 'static>() -> &'static dyn AnyMetadata {
    &MetadataType::<T>(PhantomData)
}
