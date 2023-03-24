use std::sync::{Arc, Weak};

use smallvec::SmallVec;
use tokio::sync::Mutex;

use crate::event::EventEmitter;
use crate::Event;

use super::EventReceiver;

use super::raw_channel::key::ChannelKey;
use super::raw_channel::RawChannel;

#[derive(Default, Clone)]
pub struct EventDispatcher(pub(super) Arc<Mutex<Vec<Weak<RawChannel>>>>);

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
        let mut receiver = EventReceiver::new();
        receiver.join(self).await;
        receiver
    }

    pub(super) async fn add_receiver(&self, channel: Weak<RawChannel>) {
        self.0.lock().await.push(channel);
    }

    pub(super) async fn emit<E>(&self, event: &E, key: &dyn ChannelKey)
    where
        E: Event + Clone,
    {
        let mut guard = self.0.lock().await;
        let mut clean_list: SmallVec<[usize; 4]> = SmallVec::new();

        for (index, chan) in guard.iter().enumerate() {
            match chan.upgrade() {
                Some(arc) => arc.write(event.clone(), key).await,
                None => clean_list.push(index),
            }
        }

        for index in clean_list {
            guard.remove(index);
        }
    }
}
