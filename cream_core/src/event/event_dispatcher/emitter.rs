use std::{any::Any, sync::Arc};

use crate::Event;

#[derive(Clone)]
pub struct EventEmitter(Option<Arc<dyn Fn(&dyn Any) + Send + Sync>>);

impl EventEmitter {
    pub(super) fn new_keyed(send_fn: Arc<dyn Fn(&dyn Any) + Send + Sync>) -> Self {
        Self(Some(send_fn))
    }

    pub const fn new_empty() -> Self {
        Self(None)
    }

    pub fn emit<E: Event>(&self, event: &E) {
        if let Some(send_fn) = &self.0 {
            send_fn(event);
        }
    }
}
