pub mod application;
pub mod runtime;
pub mod window;

pub use application::AppWindow;
pub use runtime::start_runtime;
pub use skia_safe;
pub use winit;

pub(crate) type WinitWindow = winit::window::Window;

pub use runtime::TOKIO_RT;

// only for export, shouldn't use in crate, which may cause confusion
pub type WindowEvent = winit::event::WindowEvent<'static>;
