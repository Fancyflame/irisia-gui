use async_trait::async_trait;
pub use event_dispatcher::{receive::EventReceive, EventDispatcher};
pub use metadata::EventMetadata;

use self::event_dispatcher::lock::EventDispatcherLock;

pub mod event_dispatcher;
pub mod metadata;
pub mod standard;

pub trait Event: Sized + Send + Unpin + Clone + 'static {}

#[async_trait]
pub trait SubEvent {
    async fn handle(ed: &mut EventReceiver) -> Self;
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

    pub async fn recv_sys<E: Event>(&mut self) -> E {
        match self {
            Self::EventDispatcher(ed) => ed.recv_sys().await,
            Self::Lock(lock) => lock.recv_sys().await,
        }
    }
}
