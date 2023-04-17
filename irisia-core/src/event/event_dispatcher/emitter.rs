use crate::event::{element_handle::ElementHandle, standard::ElementCreated};

use super::EventDispatcher;

pub struct CreatedEventEmitter<'a, K>(Option<Inner<'a, K>>);

struct Inner<'a, K> {
    send_to: &'a EventDispatcher,
    key: K,
}

impl<'a, K> CreatedEventEmitter<'a, K>
where
    K: Clone + Unpin + Send + 'static,
{
    pub(crate) fn new(event_dispatcher: &'a EventDispatcher, key: K) -> Self {
        Self(Some(Inner {
            send_to: event_dispatcher,
            key,
        }))
    }

    pub(crate) fn emit(self, eh: &ElementHandle) {
        if let Some(inner) = self.0 {
            inner.send_to.emit_sys(ElementCreated {
                result: eh.clone(),
                key: inner.key,
            });
        }
    }
}

impl CreatedEventEmitter<'static, ()> {
    pub const fn new_empty() -> Self {
        Self(None)
    }
}
