pub use event_dispatcher::{receive::EventReceive, EventDispatcher};
pub use metadata::EventMetadata;

pub mod event_dispatcher;
pub mod metadata;
pub mod standard;

pub trait Event: Send + Clone + Unpin + 'static {}
