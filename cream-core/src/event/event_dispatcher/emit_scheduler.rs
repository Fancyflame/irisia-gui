use std::collections::VecDeque;

use tokio::task::JoinHandle;

use crate::{event::EventMetadata, Event};

pub(super) struct EmitScheduler {
    executor: Option<JoinHandle<()>>,
    wait_list: VecDeque<QueuedEvent>,
}

impl EmitScheduler {
    pub fn new() -> Self {
        EmitScheduler {
            executor: None,
            wait_list: Default::default(),
        }
    }
}

struct QueuedEvent {
    event: Box<dyn FnOnce() + Send + 'static>,
}

impl super::EventDispatcher {
    pub(super) fn emit_raw<E: Event>(&self, event: E, metadata: EventMetadata) {
        let mut guard = self.0.lock().unwrap();

        let this = self.clone();
        if guard.emit_sch.executor.is_none() {
            // emit the event without delay
            if let Some(item) = guard.item_map.get() {
                item.finish(event, metadata);
            }

            let handle = tokio::spawn(async move {
                loop {
                    tokio::task::yield_now().await;
                    let mut guard = this.0.lock().unwrap();
                    match guard.emit_sch.wait_list.pop_front() {
                        Some(f) => {
                            // do not delete this `drop`, or will cause a deadlock
                            drop(guard);
                            (f.event)();
                        }
                        None => {
                            guard.emit_sch.executor = None;
                            break;
                        }
                    }
                }
            });

            guard.emit_sch.executor = Some(handle);
        } else {
            guard.emit_sch.wait_list.push_back(QueuedEvent {
                event: Box::new(move || {
                    if let Some(item) = this.0.lock().unwrap().item_map.get() {
                        item.finish(event, metadata);
                    }
                }),
            });
        }
    }
}
