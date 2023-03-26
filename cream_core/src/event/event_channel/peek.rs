use std::{any::TypeId, collections::VecDeque, io::Read};

use tokio::sync::MutexGuard;

use crate::{event::Data, Event};

use super::raw_channel::header::{self, Header};

pub struct Peek<'a> {
    // hold the lock, preventing the headers from invalidation
    buffer: MutexGuard<'a, VecDeque<u8>>,
    header: Header,
    key_header: Header,
}

impl<'pk> Peek<'pk> {
    pub(crate) unsafe fn from_buffer(buffer: MutexGuard<'pk, VecDeque<u8>>) -> Self {
        let mut reader = {
            let (a, b) = buffer.as_slices();
            a.chain(b)
        };

        Peek {
            header: header::from_bytes(&mut reader),
            key_header: header::from_bytes(&mut reader),
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

    pub fn upgrade(self) -> Data<'pk> {
        unsafe { Data::from_buffer(self.buffer) }
    }
}
