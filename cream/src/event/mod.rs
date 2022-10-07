use std::any::Any;

pub(crate) mod callback;
pub mod event_flow;
pub(crate) mod event_target;
pub(crate) mod event_target_finder;

pub use event_flow::EventFlow;

pub trait Event: Any + Copy {
    const AREA_FIND: bool;
    type Arg;
}
