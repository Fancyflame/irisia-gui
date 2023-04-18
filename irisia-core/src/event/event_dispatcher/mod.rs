use self::{emit_scheduler::EmitScheduler, emitter::CreatedEventEmitter, item_map::ItemMap};
use crate::Event;
use std::sync::{Arc, Mutex as StdMutex, Weak};

use super::{element_handle::ElementHandle, standard::ElementCreated, EventMetadata, EventReceive};

mod emit_scheduler;
pub mod emitter;
mod extension;
mod item_map;
pub mod receive;

#[derive(Clone)]
pub struct EventDispatcher(Arc<StdMutex<EventDispatcherInner>>);

#[derive(Clone)]
pub struct WeakEventDispatcher(Weak<StdMutex<EventDispatcherInner>>);

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

    pub fn recv_element_created<K>(&self, key: K) -> CreatedEventEmitter<K>
    where
        K: Clone + Unpin + Send + 'static,
    {
        CreatedEventEmitter::new(self, key)
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

    pub async fn get_element<K>(&self) -> ElementCreated<K>
    where
        K: Clone + Unpin + Send + 'static,
    {
        self.get_element_checked(|_: &K| true).await
    }

    pub async fn get_element_eq<K>(&self, key: &K) -> ElementHandle
    where
        K: Eq + Clone + Unpin + Send + 'static,
    {
        self.get_element_checked(|key_recv: &K| key_recv == key)
            .await
            .result
    }

    pub async fn get_element_checked<K, F>(&self, check: F) -> ElementCreated<K>
    where
        K: Clone + Unpin + Send + 'static,
        F: Fn(&K) -> bool,
    {
        loop {
            let (result, metadata) = self.recv::<ElementCreated<K>>().await;
            if metadata.is_system_event && check(&result.key) {
                return result;
            }
        }
    }

    pub fn downgrade(&self) -> WeakEventDispatcher {
        WeakEventDispatcher(Arc::downgrade(&self.0))
    }

    pub(crate) fn as_ptr(&self) -> *const () {
        Arc::as_ptr(&self.0) as _
    }

    pub fn is_same(&self, other: &EventDispatcher) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl WeakEventDispatcher {
    pub fn upgrade(&self) -> Option<EventDispatcher> {
        self.0.upgrade().map(|arc| EventDispatcher(arc))
    }

    pub fn is_same(&self, other: &Self) -> bool {
        self.0.ptr_eq(&other.0)
    }
}
