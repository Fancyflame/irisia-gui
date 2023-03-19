pub use event_channel::{
    data::Data,
    emitter::EventEmitter,
    getter::{EventChanGetter, EventReceiver},
    setter::EventChanSetter,
};
pub use event_flow::EventFlow;

pub mod event_channel;
pub mod event_flow;

pub trait Event: Send + 'static {}
