use std::{any::Any, sync::Arc};

use crate::Event;

type SendFn = Arc<dyn Fn(&dyn Any) + Send + Sync>;

#[derive(Clone)]
pub struct EventEmitter(Option<SendFn>);

impl EventEmitter {
    pub(super) fn new_keyed(send_fn: SendFn) -> Self {
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
