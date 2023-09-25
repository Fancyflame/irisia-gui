use std::{ops::Deref, sync::Arc};

use crate::WinitWindow;

pub use winit::window::WindowBuilder;

pub use self::close_handle::CloseHandle;

mod close_handle;
mod create;

#[derive(Clone)]
pub struct RawWindowHandle {
    pub raw_window: Arc<WinitWindow>,
    pub close_handle: CloseHandle,
}

impl Deref for RawWindowHandle {
    type Target = WinitWindow;
    fn deref(&self) -> &Self::Target {
        &self.raw_window
    }
}
