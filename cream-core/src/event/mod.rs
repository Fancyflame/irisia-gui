pub use event_dispatcher::{emitter::EventEmitter, receive::EventReceive, EventDispatcher};
pub use event_flow::EventFlow;

pub mod event_dispatcher;
pub mod event_flow;
pub mod standard;

pub trait Event: Send + Clone + Unpin + 'static {}

/// Can be received as key. It declares that this event is a
/// element event.
#[derive(Clone)]
pub struct ElementEvent(pub(crate) ());
