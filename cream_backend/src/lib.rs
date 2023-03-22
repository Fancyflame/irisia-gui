pub mod application;
mod render_window;
pub mod runtime;
pub mod window_handle;

pub use application::AppWindow;
pub use runtime::start_runtime;
pub use skia_safe;
pub use winit;

pub type WinitWindow = winit::window::Window;

// only for export, shouldn't use in crate, which may cause confusion
pub type WindowEvent = winit::event::WindowEvent<'static>;
