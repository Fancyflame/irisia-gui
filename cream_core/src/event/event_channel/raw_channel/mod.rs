use std::collections::VecDeque;

use tokio::sync::{Mutex, Notify};

use self::{header::Header, key::ChannelKey};

use super::data::Data;

pub(crate) mod header;
pub(crate) mod key;

pub(crate) struct RawChannel {
    ring_buffer: Mutex<VecDeque<u8>>,
    notify: Notify,
}

impl RawChannel {
    pub fn new() -> Self {
        RawChannel {
            ring_buffer: Mutex::new(VecDeque::new()),
            notify: Notify::new(),
        }
    }

    pub async fn write<T>(&self, value: T, key: &dyn ChannelKey)
    where
        T: Send + 'static,
    {
        let mut queue = self.ring_buffer.lock().await;
        unsafe {
            header::to_bytes(Header::of::<T>(), &mut *queue);
            key.as_key_write_header(&mut queue);
            header::to_bytes(value, &mut *queue);
            key.as_key_write_self_cloned(&mut queue);
        }
        self.notify.notify_waiters();
    }

    pub async fn read(&self) -> Data {
        loop {
            let queue = self.ring_buffer.lock().await;
            if queue.is_empty() {
                drop(queue);
                self.notify.notified().await;
                continue;
            }

            unsafe {
                break Data::from_buffer(queue);
            }
        }
    }
}

impl Drop for RawChannel {
    fn drop(&mut self) {
        let mut queue = self.ring_buffer.get_mut();
        while !queue.is_empty() {
            unsafe {
                let header: Header = header::from_bytes(&mut queue);
                let key_header: Header = header::from_bytes(&mut queue);
                (header.drop_fn)(&mut queue);
                (key_header.drop_fn)(&mut queue);
            }
        }
    }
}
