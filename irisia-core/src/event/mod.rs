pub use event_dispatcher::{receive::EventReceive, EventDispatcher};
pub use metadata::EventMetadata;

pub mod element_handle;
pub mod event_dispatcher;
pub mod metadata;
pub mod standard;

pub trait Event: Send + Clone + Unpin + 'static {}

/// Can be received as key. It declares that this event is a
/// element event.
#[derive(Clone)]
pub struct ElementEvent(pub(crate) ());
