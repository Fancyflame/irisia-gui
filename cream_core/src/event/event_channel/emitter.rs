use crate::event::Event;

use super::{event_dispatcher::EventDispatcher, raw_channel::key::ChannelKey, EventReceiver};

#[derive(Clone)]
pub struct EventEmitter(Option<Inner>);

struct Inner {
    key: Box<dyn ChannelKey>,
    channels: EventDispatcher,
}

impl EventEmitter {
    pub(super) fn join<K>(key: K, channels: EventDispatcher) -> Self
    where
        K: Clone + Send + Sync + 'static,
    {
        Self(Some(Inner {
            key: Box::new(key) as _,
            channels,
        }))
    }

    pub const fn new_no_receiver() -> Self {
        EventEmitter(None)
    }

    pub async fn emit<E>(&self, event: &E)
    where
        E: Event + Clone,
    {
        if let Some(Inner { key, channels }) = self.0.as_ref() {
            channels.emit(event, key.as_ref()).await;
        }
    }

    pub async fn get_receiver(&self) -> EventReceiver {
        match &self.0 {
            Some(Inner { channels, .. }) => channels.get_receiver().await,
            None => EventReceiver::new(),
        }
    }
}

impl Clone for Inner {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone_boxed(),
            channels: self.channels.clone(),
        }
    }
}
