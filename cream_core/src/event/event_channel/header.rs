use std::{
    any::TypeId,
    collections::VecDeque,
    io::{Read, Write},
    mem::{size_of, MaybeUninit},
};

#[derive(Clone, Copy)]
pub struct Header {
    pub type_id: TypeId,
    pub drop_fn: unsafe fn(&mut VecDeque<u8>),
}

impl Header {
    pub fn of<T>() -> Self
    where
        T: 'static,
    {
        unsafe fn make_drop_fn<T: 'static>(ring_buffer: &mut VecDeque<u8>) {
            from_bytes::<T, _>(ring_buffer);
        }

        Header {
            type_id: TypeId::of::<T>(),
            drop_fn: make_drop_fn::<T>,
        }
    }
}

pub unsafe fn from_bytes<T, R>(mut reader: R) -> T
where
    T: 'static,
    R: Read,
{
    let mut value = MaybeUninit::uninit();
    let buffer = std::slice::from_raw_parts_mut(value.as_mut_ptr() as *mut u8, size_of::<T>());
    reader.read_exact(buffer).expect("inner error: broken data");
    value.assume_init()
}

pub unsafe fn to_bytes<T, W>(value: T, mut writer: W)
where
    T: 'static,
    W: Write,
{
    let value = std::slice::from_raw_parts(&value as *const T as *const u8, size_of::<T>());
    writer
        .write_all(value)
        .expect("inner error: cannot write data");
    std::mem::forget(value);
}
