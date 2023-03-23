use std::sync::{Arc, Weak};

use tokio::sync::Mutex;

use crate::event::EventEmitter;
use crate::Event;

use super::EventReceiver;

use super::raw_channel::key::ChannelKey;
use super::raw_channel::RawChannel;

#[derive(Default, Clone)]
pub struct EventDispatcher(Arc<Mutex<Vec<Weak<RawChannel>>>>);

impl EventDispatcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_emitter(&self) -> EventEmitter {
        EventEmitter::join((), self.clone())
    }

    pub async fn get_emitter_keyed<K>(&self, key: K) -> EventEmitter
    where
        K: Clone + Send + Sync + 'static,
    {
        EventEmitter::join(key, self.clone())
    }

    pub async fn get_receiver(&self) -> EventReceiver {
        let chan = Arc::new(RawChannel::new());
        self.0.lock().await.push(Arc::downgrade(&chan));
        EventReceiver::join(chan)
    }

    pub(super) async fn emit<E>(&self, event: &E, key: &dyn ChannelKey)
    where
        E: Event + Clone,
    {
        let mut guard = self.0.lock().await;
        let mut needs_clear = false;

        for chan in guard.iter() {
            match chan.upgrade() {
                Some(arc) => arc.write(event.clone(), key).await,
                None => needs_clear = true,
            }
        }

        if needs_clear {
            guard.retain(|x| x.upgrade().is_some());
        }
    }
}
