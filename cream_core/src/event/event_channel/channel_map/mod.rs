use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use self::{getter::EventChanGetter, setter::EventChanSetter};

use super::event_dispatcher::EventDispatcher;

pub mod getter;
pub mod setter;

pub(crate) fn channel_map(global: EventDispatcher) -> (EventChanSetter, EventChanGetter) {
    let map = ChannelMap::new();
    (
        EventChanSetter {
            named_channel: map.clone(),
        },
        EventChanGetter {
            global,
            named_channel: map,
        },
    )
}

#[derive(Clone, Default)]
struct ChannelMap(Arc<RwLock<HashMap<&'static str, EventDispatcher>>>);

impl ChannelMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_dispatcher(&self, name: &'static str) -> EventDispatcher {
        let read_guard = self.0.read().await;
        if let Some(ed) = read_guard.get(name) {
            return ed.clone();
        }

        drop(read_guard);
        let mut write = self.0.write().await;
        let mut ed = EventDispatcher::new();

        // In rare cases, between read guard released
        // and write guard aquired, the setter half
        // setted new listener list. Then insert it
        // back to the list.
        if let Some(old) = write.insert(name, ed.clone()) {
            ed = old.clone();
            write.insert(name, old);
        }

        return ed;
    }
}
