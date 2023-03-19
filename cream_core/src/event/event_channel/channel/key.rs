use std::collections::VecDeque;

use crate::event::event_channel::header::{self, Header};

pub(crate) unsafe trait ChannelKey: Send + Sync + 'static {
    unsafe fn as_key_write_header(&self, writer: &mut VecDeque<u8>);
    unsafe fn as_key_write_self_cloned(&self, writer: &mut VecDeque<u8>);
}

unsafe impl<T: Clone + Send + Sync + 'static> ChannelKey for T {
    unsafe fn as_key_write_header(&self, writer: &mut VecDeque<u8>) {
        header::to_bytes(Header::of::<Self>(), writer);
    }

    unsafe fn as_key_write_self_cloned(&self, writer: &mut VecDeque<u8>) {
        header::to_bytes(self.clone(), writer);
    }
}
