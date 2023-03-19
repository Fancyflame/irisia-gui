use std::sync::Arc;

use crate::event::Event;

use super::channel::{key::ChannelKey, RawChannel};

pub struct EventEmitter(Option<Inner>);

struct Inner {
    key: Box<dyn ChannelKey>,
    channel: Arc<RawChannel>,
}

impl EventEmitter {
    pub(crate) fn new<K>(key: K, channel: Arc<RawChannel>) -> Self
    where
        K: Clone + Send + Sync + 'static,
    {
        Self(Some(Inner {
            key: Box::new(key),
            channel,
        }))
    }

    pub const fn new_empty() -> Self {
        EventEmitter(None)
    }

    pub async fn emit<E>(&self, event: E)
    where
        E: Event,
    {
        if let Some(Inner { key, channel }) = self.0.as_ref() {
            channel.write(event, key.as_ref()).await;
        }
    }
}
