pub mod application;
pub mod runtime;
pub mod window;

pub use application::Application;
pub use skia_safe::{Canvas, Paint};

pub(crate) type WinitWindow = winit::window::Window;
