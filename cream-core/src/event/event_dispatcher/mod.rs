use self::{
    emitter::EventEmitter,
    item::{Item, Visitor},
};
use crate::Event;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::{standard::EventDispatcherCreated, EventReceive};

pub mod emitter;
mod item;
pub mod receive;

#[derive(Clone)]
pub struct RecvOnly(EventDispatcher);

#[derive(Clone)]
pub struct EventDispatcher(Arc<Mutex<EventDispatcherInner>>);

struct EventDispatcherInner {
    item_map: HashMap<(TypeId, TypeId), Box<dyn Visitor>>,
}

impl EventDispatcherInner {
    fn get_item<E, K>(&mut self) -> &mut Item<E, K>
    where
        E: Event,
        K: Send + Clone + 'static,
    {
        self.item_map
            .get_mut(&(TypeId::of::<E>(), TypeId::of::<K>()))
            .expect("inner error: key-item pair not found while a receiving task still alive")
            .as_any_mut()
            .downcast_mut::<Item<E, K>>()
            .expect("inner error: cannot downcast to item")
    }
}

impl EventDispatcher {
    pub fn new() -> Self {
        EventDispatcher(Arc::new(Mutex::new(EventDispatcherInner {
            item_map: HashMap::new(),
        })))
    }

    pub fn emit<E, K>(&self, event: &E, key: &K)
    where
        E: Event,
        K: Clone + Send + Unpin + 'static,
    {
        let mut guard = self.0.lock().unwrap();
        if let Some(item) = guard
            .item_map
            .get_mut(&(TypeId::of::<E>(), TypeId::of::<K>()))
        {
            item.as_any_mut()
                .downcast_mut::<Item<E, K>>()
                .unwrap()
                .finish(event, key);
        }
    }

    pub fn emitter<K>(&self, key: K) -> EventEmitter
    where
        K: Clone + Send + Sync + Unpin + 'static,
    {
        let dispatcher = self.clone();

        let send_fn = move |ev: &dyn Any| {
            if let Some(item) = dispatcher
                .0
                .lock()
                .unwrap()
                .item_map
                .get_mut(&(ev.type_id(), TypeId::of::<K>()))
            {
                item.emit(ev, &key);
            }
        };

        EventEmitter::new_keyed(Arc::new(send_fn))
    }

    pub fn recv<E, K>(&self) -> EventReceive<E, K>
    where
        E: Event,
        K: Clone + Send + Unpin + 'static,
    {
        let key = (TypeId::of::<E>(), TypeId::of::<K>());
        let mut guard = self.0.lock().unwrap();

        let item: &mut Item<E, K> = match guard.item_map.get_mut(&key) {
            Some(visitor) => visitor.as_any_mut().downcast_mut().unwrap(),
            None => {
                guard
                    .item_map
                    .insert(key, Box::new(Item::<E, K>::new()) as _);
                guard.get_item()
            }
        };

        EventReceive::new(self, item.register())
    }

    pub async fn recv_event_dispatcher<K>(&self, key: &K) -> EventDispatcher
    where
        K: Clone + Eq + Send + Unpin + 'static,
    {
        loop {
            let (ed, key2) = self.recv::<EventDispatcherCreated, K>().await;
            if *key == key2 {
                return ed.0;
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
    pub fn recv<E, K>(&self) -> EventReceive<E, K>
    where
        E: Event,
        K: Clone + Send + Unpin + 'static,
    {
        self.0.recv()
    }
}
