pub mod application;
pub mod runtime;
pub mod window;

pub use application::AppWindow;
pub use skia_safe::{Canvas, Paint};

pub(crate) type WinitWindow = winit::window::Window;

// only for export, shouldn't use in crate, which may cause confusion
pub type WindowEvent = winit::event::WindowEvent<'static>;
