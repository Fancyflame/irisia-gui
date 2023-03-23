use std::{any::Any, collections::VecDeque};

use super::header::{self, Header};

pub(crate) unsafe trait ChannelKey: Send + Sync + 'static {
    unsafe fn as_key_write_header(&self, writer: &mut VecDeque<u8>);
    unsafe fn as_key_write_self_cloned(&self, writer: &mut VecDeque<u8>);
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

unsafe impl<T: Clone + Send + Sync + 'static> ChannelKey for T {
    unsafe fn as_key_write_header(&self, writer: &mut VecDeque<u8>) {
        header::to_bytes(Header::of::<Self>(), writer);
    }

    unsafe fn as_key_write_self_cloned(&self, writer: &mut VecDeque<u8>) {
        header::to_bytes(self.clone(), writer);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as _
    }
}
