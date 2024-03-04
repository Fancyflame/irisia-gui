use irisia_backend::window_handle::CloseHandle;
use irisia_backend::winit::event::WindowEvent;

use crate as irisia;
use crate::primitive::{Pixel, Point, Region};
use crate::Event;

//pub mod window_event;

impl Event for WindowEvent {}

/// Declares the element won't be used by the origin structure anymore,
/// but may not dropped immediately due to strong references of `Arc`.
/// This event will be emitted only
/// once, when received, runtimes of this element should handle quiting.
#[derive(Event, Clone, Copy)]
pub struct ElementAbandoned;

#[derive(Event, Clone, Copy)]
pub struct Blured;

#[derive(Event, Clone, Copy)]
pub struct Focused;

#[derive(Event, Clone)]
pub struct PointerDown {
    pub is_current: bool,
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
    pub delta: (Pixel, Pixel),
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

#[derive(Event, Clone, Copy)]
pub struct CloseRequested(pub CloseHandle);

#[derive(Event, Clone, Copy)]
pub struct WindowDestroyed;

#[derive(Event, Clone, Copy)]
pub struct DrawRegionChanged {
    pub region: Region,
}
