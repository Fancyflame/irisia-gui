use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::event::{EventChanGetter, EventReceiver};

use super::{channel::RawChannel, emitter::EventEmitter, GLOBAL_EVENT_RECEIVER_NAME};

pub(super) type InnerMap = Arc<RwLock<HashMap<&'static str, Arc<RawChannel>>>>;

pub struct EventChanSetter {
    named_channel: InnerMap,
}

impl EventChanSetter {
    pub(crate) fn channel(global: EventReceiver) -> (Self, EventChanGetter) {
        let chan: Arc<RwLock<HashMap<&'static str, Arc<RawChannel>>>> = Default::default();
        (
            EventChanSetter {
                named_channel: chan.clone(),
            },
            EventChanGetter {
                global,
                named_channel: chan,
            },
        )
    }

    pub fn to_emitter(&self, name: &'static str) -> EventEmitter {
        self.to_emitter_with_key(name, ())
    }

    pub fn to_emitter_with_key<K>(&self, name: &'static str, key: K) -> EventEmitter
    where
        K: Clone + Send + Sync + 'static,
    {
        if name == GLOBAL_EVENT_RECEIVER_NAME {
            panic!("channel name cannot be `{GLOBAL_EVENT_RECEIVER_NAME}`, which is preserved as global event channel");
        }

        let chan = match self.named_channel.blocking_read().get(name) {
            Some(chan) => chan.clone(),
            None => {
                let chan = Arc::new(RawChannel::new());
                let mut write = self.named_channel.blocking_write();
                write.insert(name, chan.clone());
                chan
            }
        };
        EventEmitter::new(key, chan)
    }
}
