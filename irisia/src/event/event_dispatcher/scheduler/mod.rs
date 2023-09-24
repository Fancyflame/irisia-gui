use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use tokio::task::JoinHandle;

use crate::{event::EventMetadata, Event};

use super::maybe_confirmed::{AllConfirmedPermits, MaybeConfirmed};
use stock::EventListenerStock;

pub(super) mod stock;

pub(super) struct EmitScheduler {
    stock: EventListenerStock,
    executor: Option<JoinHandle<()>>,
    event_queue: VecDeque<QueuedEvent>,
    wait_lock: Arc<MaybeConfirmed>,
}

struct QueuedEvent {
    #[allow(clippy::type_complexity)]
    event: Box<dyn FnOnce(&mut EmitScheduler, AllConfirmedPermits) + Send + 'static>,
}

impl EmitScheduler {
    pub fn new() -> Self {
        let arc = Arc::new(MaybeConfirmed::new());
        EmitScheduler {
            stock: EventListenerStock::new(arc.clone()),
            executor: None,
            event_queue: Default::default(),
            wait_lock: arc,
        }
    }

    pub(super) fn emit_raw<E: Event>(this: &Arc<Mutex<Self>>, event: E, metadata: EventMetadata) {
        let mut guard = this.lock().unwrap();

        // there is existing executor
        if guard.executor.is_some() {
            guard.event_queue.push_back(QueuedEvent {
                event: Box::new(move |scheduler, permits| {
                    if let Some(item) = scheduler.stock.get() {
                        item.finish(event, metadata, permits);
                    }
                }),
            });
            return;
        }

        // there is no existing executor, but can execute immediately
        let guard_ref = &mut *guard;
        if let Ok(permits) = guard_ref.wait_lock.try_all_confirmed() {
            if let Some(item) = guard_ref.stock.get() {
                item.finish(event, metadata, permits);
            }
            return;
        }

        // there is no existing executor, also has lock held
        guard_ref.spwan_executor(this, event, metadata);
    }

    fn spwan_executor<E: Event>(
        &mut self,
        this: &Arc<Mutex<Self>>,
        event: E,
        metadata: EventMetadata,
    ) {
        let this = this.clone();
        let wait_lock = self.wait_lock.clone();
        let handle = tokio::spawn(async move {
            let permits = wait_lock.all_confirmed().await;
            let mut next_event = {
                let mut guard = this.lock().unwrap();
                if let Some(item) = guard.stock.get() {
                    item.finish(event, metadata, permits);
                }

                match guard.get_event() {
                    Some(ev) => ev,
                    None => {
                        guard.executor = None;
                        return;
                    }
                }
            };

            loop {
                let permits = wait_lock.all_confirmed().await;
                let mut guard = this.lock().unwrap();
                match guard.event_queue.pop_front() {
                    Some(next) => {
                        let current_event = std::mem::replace(&mut next_event, next);
                        (current_event.event)(&mut guard, permits);
                    }
                    None => {
                        (next_event.event)(&mut guard, permits);
                        guard.executor = None;
                        return;
                    }
                }
            }
        });

        self.executor = Some(handle);
    }

    fn get_event(&mut self) -> Option<QueuedEvent> {
        let event = self.event_queue.pop_front();
        if event.is_none() {}
        event
    }

    pub(super) fn stock(&mut self) -> &mut EventListenerStock {
        &mut self.stock
    }

    pub(super) fn wait_lock(&self) -> &Arc<MaybeConfirmed> {
        &self.wait_lock
    }
}
