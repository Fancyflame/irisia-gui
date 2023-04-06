use tokio::{
    sync::{Mutex as AsyncMutex, MutexGuard as AsyncMutexGuard},
    task::JoinHandle,
};

use self::{emitter::CreatedEventEmitter, item::Item};
use crate::Event;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, Mutex as StdMutex, MutexGuard as StdMutexGuard},
};

use super::{standard::EventDispatcherCreated, EventMetadata, EventReceive};

pub mod emitter;
mod item;
pub mod receive;

#[derive(Clone)]
pub struct RecvOnly(EventDispatcher);

#[derive(Clone)]
pub struct EventDispatcher(Arc<EventDispatcherInner>);

type ItemMap = HashMap<TypeId, Box<dyn Any + Send>>;

struct EventDispatcherInner {
    item_map: StdMutex<ItemMap>,
    wait_yield: AsyncMutex<Option<JoinHandle<()>>>,
}

fn get_item<'a, E: Event>(map: &'a mut StdMutexGuard<ItemMap>) -> Option<&'a mut Item<E>> {
    map.get_mut(&TypeId::of::<E>()).map(|any| {
        any.downcast_mut()
            .expect("inner error: cannot downcast to item")
    })
}

fn get_exist_item<'a, E: Event>(map: &'a mut StdMutexGuard<ItemMap>) -> &'a mut Item<E> {
    match get_item(map) {
        Some(item) => item,
        None => {
            panic!("inner error: item not found");
        }
    }
}

impl EventDispatcher {
    pub fn new() -> Self {
        EventDispatcher(Arc::new(EventDispatcherInner {
            item_map: Default::default(),
            wait_yield: AsyncMutex::new(None),
        }))
    }

    pub fn emit(&self, event: impl Event) {
        self.emit_raw(event, EventMetadata::new())
    }

    pub(crate) fn emit_sys(&self, event: impl Event) {
        self.emit_raw(event, EventMetadata::new_sys())
    }

    fn emit_raw<E: Event>(&self, event: E, metadata: EventMetadata) {
        let emit =
            move |this: &EventDispatcher,
                  yield_guard: &mut AsyncMutexGuard<'_, Option<JoinHandle<()>>>| {
                yield_guard.replace(tokio::spawn(tokio::task::yield_now()));
                let mut guard = this.0.item_map.lock().unwrap();
                if let Some(item) = get_item(&mut guard) {
                    item.finish(event, metadata);
                }
            };

        // fast emit
        if let Ok(mut yield_guard) = self.0.wait_yield.try_lock() {
            let finished = match &*yield_guard {
                Some(last) => last.is_finished(),
                None => true,
            };

            if finished {
                emit(self, &mut yield_guard);
                return;
            }
        }

        let this = self.clone();
        tokio::spawn(async move {
            let mut yield_guard = this.0.wait_yield.lock().await;

            // we can `take().unwrap()` here, because we checked it must be `Some` before
            yield_guard.take().unwrap().await.unwrap();

            emit(&this, &mut yield_guard);
        });
    }

    pub fn recv<E: Event>(&self) -> EventReceive<E> {
        let tid = TypeId::of::<E>();
        let mut guard = self.0.item_map.lock().unwrap();

        let id;
        match get_item::<E>(&mut guard) {
            Some(item) => id = item.register(),
            None => {
                let mut item = Item::<E>::new();
                id = item.register();
                guard.insert(tid, Box::new(item) as _);
            }
        }

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
