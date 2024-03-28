use std::future::Future;

pub use self::{
    event_dispatcher::{receive::EventReceive, EventDispatcher},
    listen::Listen,
    metadata::EventMetadata,
};

use self::event_dispatcher::lock::EventDispatcherLock;

pub mod event_dispatcher;
mod listen;
pub mod metadata;
pub mod standard;

pub trait Event: Sized + Send + Unpin + Clone + 'static {}

pub trait SubEvent: Sized {
    fn handle(ed: &mut EventReceiver) -> impl Future<Output = Self>;
}

pub enum EventReceiver<'a> {
    EventDispatcher(&'a EventDispatcher),
    Lock(EventDispatcherLock<'a>),
}

impl EventReceiver<'_> {
    pub fn recv<E: Event>(&mut self) -> EventReceive<E> {
        match self {
            Self::EventDispatcher(ed) => ed.recv(),
            Self::Lock(lock) => lock.recv(),
        }
    }

    pub async fn recv_trusted<E: Event>(&mut self) -> E {
        match self {
            Self::EventDispatcher(ed) => ed.recv_trusted().await,
            Self::Lock(lock) => lock.recv_trusted().await,
        }
    }
}
