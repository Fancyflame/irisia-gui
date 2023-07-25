use std::{future::Future, ops::Deref};

use tokio::task::JoinHandle;

use crate::{
    application::event_comp::global::focusing::Focusing,
    event::{EventDispatcher, EventMetadata},
    Event,
};

use self::callback::Listen;

pub mod callback;
mod closure_patch;

#[derive(Clone)]
pub struct EventHandle {
    event_dispatcher: EventDispatcher,
    focusing: Focusing,
}

impl EventHandle {
    pub(crate) fn new(ed: EventDispatcher, focusing: Focusing) -> Self {
        Self {
            event_dispatcher: ed,
            focusing,
        }
    }

    pub fn focus(&self) {
        self.focusing.focus(self.event_dispatcher.clone());
    }

    pub fn blur(&self) {
        self.focusing.blur();
    }

    pub fn blur_checked(&self) {
        self.focusing.blur_checked(&self.event_dispatcher);
    }

    pub fn on<E, F>(&self, f: F) -> JoinHandle<()>
    where
        E: Event,
        F: FnMut(E, EventMetadata) + Send + 'static,
    {
        Listen::new(self).spawn(f)
    }

    pub fn listen(&self) -> Listen<(), (), ()> {
        Listen::new(self)
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

impl Deref for EventHandle {
    type Target = EventDispatcher;
    fn deref(&self) -> &Self::Target {
        &self.event_dispatcher
    }
}
