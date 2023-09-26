use self::{lock::EventDispatcherLock, scheduler::EmitScheduler};
use crate::Event;
use std::sync::{Arc, Mutex as StdMutex};

use super::{listen::Listen, EventMetadata, EventReceive};

mod extension;
pub mod lock;
mod maybe_confirmed;
pub mod receive;
mod scheduler;

#[derive(Clone)]
pub struct EventDispatcher(Arc<StdMutex<EmitScheduler>>);

impl EventDispatcher {
    pub fn new() -> Self {
        EventDispatcher(Arc::new(StdMutex::new(EmitScheduler::new())))
    }

    pub fn emit(&self, event: impl Event) {
        EmitScheduler::emit_raw(&self.0, event, EventMetadata::new());
    }

    pub(crate) fn emit_trusted<E: Event>(&self, event: E) {
        EmitScheduler::emit_raw(&self.0, event, EventMetadata::new_trusted());
    }

    pub fn recv<E: Event>(&self) -> EventReceive<E> {
        let id = self
            .0
            .lock()
            .unwrap()
            .stock()
            .get_or_insert::<E>()
            .register(false);
        EventReceive::new(self, id)
    }

    pub async fn recv_trusted<E: Event>(&self) -> E {
        loop {
            let (ev, metadata) = self.recv::<E>().await;
            if metadata.is_trusted_event {
                return ev;
            }
        }
    }

    pub fn listen(&self) -> Listen<Self> {
        Listen::new(self)
    }

    pub(crate) fn ptr_eq(&self, other: &EventDispatcher) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub fn is_same(&self, other: &EventDispatcher) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub fn lock(&self) -> EventDispatcherLock {
        EventDispatcherLock::from_event_dispatcher(self)
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
