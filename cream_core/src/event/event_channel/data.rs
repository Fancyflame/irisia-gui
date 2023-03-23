use std::{any::TypeId, collections::VecDeque, fmt::Debug};

use tokio::sync::MutexGuard;

use crate::event::{
    event_channel::raw_channel::header::{self, Header},
    Event,
};

pub struct Data<'a> {
    do_drop: bool,
    buffer: MutexGuard<'a, VecDeque<u8>>,
    header: Header,
    key_header: Header,
}

impl<'a> Data<'a> {
    pub(super) unsafe fn from_buffer(mut buffer: MutexGuard<'a, VecDeque<u8>>) -> Self {
        Self {
            do_drop: true,
            header: header::from_bytes(&mut *buffer),
            key_header: header::from_bytes(&mut *buffer),
            buffer,
        }
    }

    pub fn is<E>(&self) -> bool
    where
        E: Event,
    {
        self.header.type_id == TypeId::of::<E>()
    }

    pub fn key_is<K>(&self) -> bool
    where
        K: Clone + Send + 'static,
    {
        self.key_header.type_id == TypeId::of::<K>()
    }

    pub fn assume<E>(mut self) -> Result<E, Self>
    where
        E: Event,
    {
        if self.header.type_id == TypeId::of::<E>() {
            self.do_drop = false;
            unsafe {
                let value = header::from_bytes(&mut *self.buffer);
                (self.key_header.drop_fn)(&mut self.buffer);
                Ok(value)
            }
        } else {
            Err(self)
        }
    }

    pub fn assume_keyed<E, K>(mut self) -> Result<(E, K), Self>
    where
        E: Event,
        K: Clone + Send + 'static,
    {
        if self.header.type_id == TypeId::of::<E>() && self.key_header.type_id == TypeId::of::<K>()
        {
            self.do_drop = false;
            unsafe {
                Ok((
                    header::from_bytes(&mut *self.buffer),
                    header::from_bytes(&mut *self.buffer),
                ))
            }
        } else {
            Err(self)
        }
    }
}

impl Debug for Data<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{CreamData}}")
    }
}

impl Drop for Data<'_> {
    fn drop(&mut self) {
        if self.do_drop {
            unsafe {
                (self.header.drop_fn)(&mut self.buffer);
                (self.key_header.drop_fn)(&mut self.buffer);
            }
        }
    }
}
