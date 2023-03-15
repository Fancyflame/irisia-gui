pub mod event_flow;
pub mod event_state;
pub mod global_event_register;
pub mod native;

pub use event_flow::EventFlow;
pub use event_state::{build::EventListenerBuilder, proxy::EvlProxyBuilder, wrap::WrappedEvents};

pub trait Event: 'static {}
