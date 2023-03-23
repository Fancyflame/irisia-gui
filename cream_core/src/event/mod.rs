pub use event_channel::{
    channel_map::{getter::EventChanGetter, setter::EventChanSetter},
    data::Data,
    emitter::EventEmitter,
    event_dispatcher::EventDispatcher,
    receiver::EventReceiver,
};
pub use event_flow::EventFlow;

pub mod event_channel;
pub mod event_flow;
pub mod standard;

pub trait Event: Send + 'static {}
