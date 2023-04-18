use crate as irisia;
use crate::primary::Point;
use crate::Event;

use super::element_handle::ElementHandle;

pub mod window_event;

/// Declares the element won't be used by the origin structure anymore,
/// but may not dropped immediately due to strong references of `Arc`.
/// This event will be emitted only
/// once, when received, runtimes of this element should handle quiting.
#[derive(Event, Clone, Copy)]
pub struct ElementAbondoned;

#[derive(Event, Clone)]
pub struct ElementCreated<K>
where
    K: Clone + Unpin + Send + 'static,
{
    pub result: ElementHandle,
    pub key: K,
}

#[derive(Event, Clone, Copy)]
pub struct Blured;

#[derive(Event, Clone, Copy)]
pub struct Focused;

#[derive(Event, Clone)]
pub struct PointerDown {
    pub is_current: bool,
    pub is_leading: bool,
    pub position: Point,
}

#[derive(Event, Clone, Copy)]
pub struct PointerUp {
    pub is_current: bool,
    pub position: Point,
}

#[derive(Event, Clone, Copy)]
pub struct PointerMove {
    pub is_current: bool,
    pub position: Point,
}

#[derive(Event, Clone, Copy)]
pub struct PointerEntered;

#[derive(Event, Clone, Copy)]
pub struct PointerOut;

#[derive(Event, Clone, Copy)]
pub struct PointerOver;

#[derive(Event, Clone, Copy)]
pub struct PointerLeft;

#[derive(Event, Clone, Copy)]
pub struct Click {
    pub is_current: bool,
}
