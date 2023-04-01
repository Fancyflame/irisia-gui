use std::{ops::Deref, sync::Arc};

use crate::WinitWindow;

pub use winit::window::WindowBuilder;

use self::close_handle::CloseHandle;

pub mod close_handle;
pub mod create;

pub struct WindowHandle {
    raw_window: Arc<WinitWindow>,
}

impl WindowHandle {
    pub fn close(&self) {
        CloseHandle(self.id()).close();
    }
}

impl Deref for WindowHandle {
    type Target = WinitWindow;
    fn deref(&self) -> &Self::Target {
        &self.raw_window
    }
}
