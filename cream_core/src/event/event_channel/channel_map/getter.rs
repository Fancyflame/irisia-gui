use cream_macros::match_event;

use crate::event::EventReceiver;

use crate::event::event_channel::event_dispatcher::EventDispatcher;
use crate::event::standard::ElementDropped;

use super::ChannelMap;

pub const WINDOW_EVENT_CHANNEL: &str = "@window";
pub const ELEMENT_EVENT_CHANNEL: &str = "@element";

#[derive(Clone)]
pub struct EventChanGetter {
    pub(super) global: EventDispatcher,
    pub(super) named_channel: ChannelMap,
}

impl EventChanGetter {
    pub async fn get_receiver(&self, name: &'static str) -> EventReceiver {
        match (name, name.starts_with("@")) {
            (WINDOW_EVENT_CHANNEL, _) => self.global.get_receiver().await,

            (ELEMENT_EVENT_CHANNEL, _) | (_, false) => {
                self.named_channel
                    .get_dispatcher(name)
                    .await
                    .get_receiver()
                    .await
            }

            (_, true) => {
                if cfg!(debug_assertions) {
                    panic!("automatic event channel `{name}` is not support");
                } else {
                    return EventReceiver::new_empty();
                }
            }
        }
    }

    pub async fn wait_closed(&self) {
        use crate as cream_core;
        let elem = self.get_receiver("@element").await;
        loop {
            match_event! {
                elem.recv().await => {
                    ElementDropped => return,
                }
            }
        }
    }
}
