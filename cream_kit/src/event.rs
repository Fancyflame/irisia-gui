use cream_core::event::Event;

pub struct WindowResized {
    pub width: u32,
    pub height: u32,
}
pub struct PointerDown {
    pub x: f32,
    pub y: f32,
}

impl Event for WindowResized {}
