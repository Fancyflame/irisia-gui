use self::{emit_scheduler::EmitScheduler, emitter::CreatedEventEmitter, item_map::ItemMap};
use crate::Event;
use std::sync::{Arc, Mutex as StdMutex};

use super::{standard::EventDispatcherCreated, EventMetadata, EventReceive};

mod emit_scheduler;
pub mod emitter;
mod item_map;
pub mod receive;

#[derive(Clone)]
pub struct RecvOnly(EventDispatcher);

#[derive(Clone)]
pub struct EventDispatcher(Arc<StdMutex<EventDispatcherInner>>);

struct EventDispatcherInner {
    item_map: ItemMap,
    emit_sch: EmitScheduler,
}

impl EventDispatcher {
    pub fn new() -> Self {
        EventDispatcher(Arc::new(StdMutex::new(EventDispatcherInner {
            item_map: ItemMap::new(),
            emit_sch: EmitScheduler::new(),
        })))
    }

    pub fn emit(&self, event: impl Event) {
        self.emit_raw(event, EventMetadata::new())
    }

    pub(crate) fn emit_sys(&self, event: impl Event) {
        self.emit_raw(event, EventMetadata::new_sys())
    }

    pub fn recv<E: Event>(&self) -> EventReceive<E> {
        let id = self
            .0
            .lock()
            .unwrap()
            .item_map
            .get_or_insert::<E>()
            .register();
        EventReceive::new(self, id)
    }

    pub async fn recv_sys<E: Event>(&self) -> E {
        loop {
            let (ev, metadata) = self.recv::<E>().await;
            if metadata.is_system_event {
                return ev;
            }
        }
    }

    pub fn created_event_emitter<K>(&self, key: K) -> CreatedEventEmitter<K>
    where
        K: Clone + Unpin + Send + 'static,
    {
        CreatedEventEmitter::new(self, key)
    }

    pub async fn get_element<K>(&self) -> EventDispatcherCreated<K>
    where
        K: Clone + Unpin + Send + 'static,
    {
        self.get_element_checked(|_: &K| true).await
    }

    pub async fn get_element_eq<K>(&self, key: &K) -> EventDispatcher
    where
        K: Eq + Clone + Unpin + Send + 'static,
    {
        self.get_element_checked(|key_recv: &K| key_recv == key)
            .await
            .result
    }

    pub async fn get_element_checked<K, F>(&self, check: F) -> EventDispatcherCreated<K>
    where
        K: Clone + Unpin + Send + 'static,
        F: Fn(&K) -> bool,
    {
        loop {
            let (result, metadata) = self.recv::<EventDispatcherCreated<K>>().await;
            if metadata.is_system_event && check(&result.key) {
                return result;
            }
        }
    }

    pub fn to_recv_only(&self) -> RecvOnly {
        RecvOnly(self.clone())
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl RecvOnly {
    pub fn recv<E: Event>(&self) -> EventReceive<E> {
        self.0.recv()
    }

    pub async fn recv_sys<E: Event>(&self) -> E {
        self.0.recv_sys().await
    }
}
