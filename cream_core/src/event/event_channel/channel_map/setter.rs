use crate::event::EventEmitter;

use super::ChannelMap;

#[derive(Clone)]
pub struct EventChanSetter {
    pub(super) named_channel: ChannelMap,
}

impl EventChanSetter {
    pub async fn to_emitter(&self, name: &'static str) -> EventEmitter {
        self.to_emitter_with_key(name, ()).await
    }

    pub async fn to_emitter_with_key<K>(&self, name: &'static str, key: K) -> EventEmitter
    where
        K: Clone + Send + Sync + 'static,
    {
        if name.starts_with("@") {
            panic!("channel name cannot starts with `@`, which is preserved as automatic event channel");
        }

        let ed = self.named_channel.get_dispatcher(name).await;
        EventEmitter::join(key, ed)
    }

    pub(crate) async fn to_special_event_emitter(&self, name: &'static str) -> EventEmitter {
        debug_assert!(name.starts_with("@"));
        let ed = self.named_channel.get_dispatcher(name).await;
        EventEmitter::join((), ed)
    }
}
