use async_trait::async_trait;
pub use event_dispatcher::{receive::EventReceive, EventDispatcher};
pub use metadata::EventMetadata;

pub mod event_dispatcher;
pub mod metadata;
pub mod standard;

pub trait Event: Sized + Send + Unpin + Clone + 'static {}

#[async_trait]
pub trait SubEvent {
    async fn handle(ed: &EventDispatcher) -> Self;
}
