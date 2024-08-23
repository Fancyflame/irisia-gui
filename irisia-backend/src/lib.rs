pub mod application;
mod render_window;
pub mod runtime;
pub mod window_handle;

pub use application::AppWindow;
pub use runtime::start_runtime;

pub use skia_safe;
pub use winit;

pub type WinitWindow = winit::window::Window;

#[cfg(feature = "dhat_heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
