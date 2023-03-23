use super::{data::Data, raw_channel::RawChannel};
use std::{future::pending, sync::Arc};

#[derive(Clone)]
pub struct EventReceiver(Option<Inner>);

#[derive(Clone)]
struct Inner {
    channel: Arc<RawChannel>,
}

impl EventReceiver {
    pub(super) fn join(channel: Arc<RawChannel>) -> Self {
        EventReceiver(Some(Inner { channel }))
    }

    pub(super) fn new_empty() -> Self {
        EventReceiver(None)
    }

    pub async fn recv(&self) -> Data {
        match &self.0 {
            Some(inner) => inner.channel.read().await,
            None => pending().await,
        }
    }

    pub fn never_received(&self) -> bool {
        self.0.is_none()
    }
}
