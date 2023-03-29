use crate as cream_core;
use crate::Event;

use super::EventDispatcher;

/// Declares the element won't be used anymore, but may not dropped immediately
/// due to strong references. This event will be emitted only
/// once, when received, tasks around this element should handle quiting.
#[derive(Event, Clone, Copy)]
pub struct ElementAbondoned;

#[derive(Event, Clone)]
pub struct EventDispatcherCreated(pub EventDispatcher);
