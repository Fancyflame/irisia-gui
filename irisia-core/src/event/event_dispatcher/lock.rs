use std::sync::Arc;

use crate::{event::EventReceive, Event};

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
}

impl Drop for EventDispatcherLock<'_> {
    fn drop(&mut self) {
        self.wait_lock.confirm_one();
    }
}
