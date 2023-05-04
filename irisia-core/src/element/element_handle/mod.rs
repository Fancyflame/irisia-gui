use std::{future::Future, ops::Deref};

use tokio::task::JoinHandle;

use crate::{
    application::elem_table::focus::SharedFocusing,
    event::{EventDispatcher, EventMetadata},
    Event,
};

use self::callback::On;

pub mod callback;
mod closure_patch;

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

    pub fn on<E, F>(&self, f: F) -> JoinHandle<()>
    where
        E: Event,
        F: FnMut(E, EventMetadata) + Send + 'static,
    {
        On::new(self).spawn(f)
    }

    pub fn listen(&self) -> On<(), (), ()> {
        On::new(self)
    }

    pub fn spawn<F, Ret>(&self, future: F) -> JoinHandle<Option<Ret>>
    where
        F: Future<Output = Ret> + Send + 'static,
        Ret: Send + 'static,
    {
        let ed = self.event_dispatcher.clone();
        tokio::spawn(async move { ed.cancel_on_abandoned(future).await })
    }

    pub fn event_dispatcher(&self) -> &EventDispatcher {
        &self
    }
}

impl Deref for ElementHandle {
    type Target = EventDispatcher;
    fn deref(&self) -> &Self::Target {
        &self.event_dispatcher
    }
}
