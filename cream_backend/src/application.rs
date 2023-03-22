use std::{sync::Arc, time::Duration};

use anyhow::Result;
use skia_safe::Canvas;

use crate::{window_handle::close_handle::CloseHandle, WinitWindow};

pub trait AppWindow: Send + Sync + 'static {
    fn on_create(window: &Arc<WinitWindow>, close_handle: CloseHandle) -> Result<Self>
    where
        Self: Sized;
    fn on_redraw(&mut self, canvas: &mut Canvas, size: (u32, u32), delta: Duration) -> Result<()>;
    fn on_window_event(&mut self, event: crate::WindowEvent);
}
