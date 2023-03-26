use crate::event::EventDispatcher;

use super::{data::Data, peek::Peek, raw_channel::RawChannel};
use std::{future::pending, sync::Arc};

#[derive(Clone)]
pub struct EventReceiver(Option<Inner>);

#[derive(Clone)]
struct Inner {
    channel: Arc<RawChannel>,
}

impl EventReceiver {
    pub(super) async fn join(&mut self, dispatcher: &EventDispatcher) {
        let weak = match &self.0 {
            Some(inner) => Arc::downgrade(&inner.channel),
            None => {
                let channel = Arc::new(RawChannel::new());
                let weak = Arc::downgrade(&channel);
                self.0 = Some(Inner { channel });
                weak
            }
        };

        dispatcher.add_receiver(weak).await;
    }

    pub(super) const fn new() -> Self {
        EventReceiver(None)
    }

    pub async fn recv(&self) -> Data {
        match &self.0 {
            Some(inner) => inner.channel.read().await,
            None => pending().await,
        }
    }

    pub async fn peek(&self) -> Peek {
        match &self.0 {
            Some(inner) => inner.channel.peek().await,
            None => pending().await,
        }
    }

    pub fn never_received(&self) -> bool {
        self.0.is_none()
    }
}
