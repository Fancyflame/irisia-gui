use std::any::Any;

pub mod callback;
pub mod event_flow;
pub mod event_target;

pub use event_flow::EventFlow;

pub trait Event: Any + Copy {
    const AREA_FIND: bool;
    type Arg;
}
