use std::sync::Arc;

use super::{channel::RawChannel, data::Data, setter::InnerMap, GLOBAL_EVENT_RECEIVER_NAME};

#[derive(Clone)]
pub struct EventChanGetter {
    pub(super) global: EventReceiver,
    pub(super) named_channel: InnerMap,
}

impl EventChanGetter {
    pub async fn get_receiver(&self, name: &'static str) -> EventReceiver {
        if name == GLOBAL_EVENT_RECEIVER_NAME {
            return self.global.clone();
        }

        let chan = match self.named_channel.read().await.get(name) {
            Some(chan) => chan.clone(),
            None => {
                let chan = Arc::new(RawChannel::new());
                let mut write = self.named_channel.write().await;
                write.insert(name, chan.clone());
                chan
            }
        };
        EventReceiver::new(chan)
    }
}

#[derive(Clone)]
pub struct EventReceiver(Arc<RawChannel>);

impl EventReceiver {
    pub(crate) fn new(chan: Arc<RawChannel>) -> Self {
        Self(chan)
    }

    pub async fn recv(&self) -> Data {
        self.0.read().await
    }
}
