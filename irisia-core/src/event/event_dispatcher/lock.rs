use std::sync::Arc;

use crate::{
    event::{element_handle::ElementHandle, standard::ElementCreated, EventReceive},
    Event,
};

use super::{maybe_confirmed::MaybeConfirmed, EventDispatcher};

pub struct EventDispatcherLock<'a> {
    ed: &'a EventDispatcher,
    wait_lock: Arc<MaybeConfirmed>,
}

impl<'a> EventDispatcherLock<'a> {
    pub fn from_event_dispatcher(ed: &'a EventDispatcher) -> Self {
        let wait_lock = ed.0.lock().unwrap().wait_lock().clone();
        wait_lock.cancel_one();
        EventDispatcherLock { ed, wait_lock }
    }

    pub fn recv<E: Event>(&mut self) -> EventReceive<E> {
        let id = self
            .ed
            .0
            .lock()
            .unwrap()
            .stock()
            .get_or_insert::<E>()
            .register(true);
        self.wait_lock.confirm_one();
        EventReceive::new(self.ed, id)
    }

    pub async fn recv_sys<E: Event>(&mut self) -> E {
        loop {
            let (ev, metadata) = self.recv::<E>().await;
            if metadata.is_system_event {
                return ev;
            }
        }
    }

    pub async fn get_element<K>(&mut self) -> ElementCreated<K>
    where
        K: Clone + Unpin + Send + 'static,
    {
        self.get_element_checked(|_: &K| true).await
    }

    pub async fn get_element_by_id<K>(&mut self, key: &K) -> ElementHandle
    where
        K: Eq + Clone + Unpin + Send + 'static,
    {
        self.get_element_checked(|key_recv: &K| key_recv == key)
            .await
            .result
    }

    pub async fn get_element_checked<K, F>(&mut self, check: F) -> ElementCreated<K>
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
}

impl Drop for EventDispatcherLock<'_> {
    fn drop(&mut self) {
        self.wait_lock.confirm_one();
    }
}
