use std::ops::Deref;

use crate::application::elem_table::focus::SharedFocusing;

use super::EventDispatcher;

#[derive(Clone)]
pub struct ElementHandle {
    event_dispatcher: EventDispatcher,
    focusing: SharedFocusing,
}

impl ElementHandle {
    pub(crate) fn new(ed: EventDispatcher, focusing: SharedFocusing) -> Self {
        Self {
            event_dispatcher: ed,
            focusing,
        }
    }

    pub async fn focus(&self) {
        self.focusing
            .lock()
            .await
            .focus_on(self.event_dispatcher.clone())
    }

    pub async fn blur(&self) {
        self.focusing.lock().await.blur(&self.event_dispatcher)
    }
}

impl Deref for ElementHandle {
    type Target = EventDispatcher;
    fn deref(&self) -> &Self::Target {
        &self.event_dispatcher
    }
}
