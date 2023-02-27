pub mod event_flow;
pub mod event_state;
pub(crate) mod global_register;

pub use event_flow::EventFlow;

pub use event_state::{build::EvlBuilder, proxy::EvlProxyBuilder, wrap::WrappedEvents};

pub trait Event: 'static {}
