pub use event_dispatcher::{emitter::EventEmitter, receiver::EventReceive, EventDispatcher};
pub use event_flow::EventFlow;

pub mod event_dispatcher;
pub mod event_flow;
pub mod standard;

pub trait Event: Send + Clone + Unpin + 'static {}

#[derive(Clone)]
pub struct ElementEventKey(pub(crate) ());
