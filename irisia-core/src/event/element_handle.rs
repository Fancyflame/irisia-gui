use std::{future::Future, ops::Deref};

use tokio::task::JoinHandle;

use crate::{application::elem_table::focus::SharedFocusing, event::standard::ElementAbondoned};

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
        self.focusing
            .lock()
            .await
            .blur_checked(&self.event_dispatcher)
    }

    pub fn spawn<F, Ret>(&self, future: F) -> JoinHandle<Option<Ret>>
    where
        F: Future<Output = Ret> + Send + 'static,
        Ret: Send + 'static,
    {
        let eh = self.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = eh.recv_sys::<ElementAbondoned>() => None,
                r = future => Some(r)
            }
        })
    }
}

impl Deref for ElementHandle {
    type Target = EventDispatcher;
    fn deref(&self) -> &Self::Target {
        &self.event_dispatcher
    }
}
