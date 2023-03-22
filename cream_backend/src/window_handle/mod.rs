use std::{ops::Deref, sync::Arc};

use tokio::sync::Mutex;

use crate::{AppWindow, WinitWindow};

pub use winit::window::WindowBuilder;

use self::close_handle::CloseHandle;

pub mod close_handle;
pub mod create;

pub struct WindowHandle<A: AppWindow> {
    app: Arc<Mutex<A>>,
    raw_window: Arc<WinitWindow>,
}

impl<A: AppWindow> WindowHandle<A> {
    pub fn app(&self) -> &Arc<Mutex<A>> {
        &self.app
    }

    pub fn close(&self) {
        CloseHandle(self.id()).close();
    }
}

impl<A: AppWindow> Deref for WindowHandle<A> {
    type Target = WinitWindow;
    fn deref(&self) -> &Self::Target {
        &self.raw_window
    }
}
