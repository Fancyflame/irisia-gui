use crate::event::standard::EventDispatcherCreated;

use super::EventDispatcher;

pub struct CreatedEventEmitter<'a, K>(Option<Inner<'a, K>>);

struct Inner<'a, K> {
    event_dispatcher: &'a EventDispatcher,
    key: K,
}

impl<'a, K> CreatedEventEmitter<'a, K>
where
    K: Clone + Unpin + Send + 'static,
{
    pub(super) fn new(event_dispatcher: &'a EventDispatcher, key: K) -> Self {
        Self(Some(Inner {
            event_dispatcher,
            key,
        }))
    }

    pub(crate) fn emit(self, ed: &EventDispatcher) {
        if let Some(inner) = self.0 {
            inner.event_dispatcher.emit_sys(EventDispatcherCreated {
                result: ed.clone(),
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
