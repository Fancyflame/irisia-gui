use std::{sync::Arc, time::Duration};

use anyhow::Result;
use skia_safe::Canvas;

use crate::window::Window;

pub trait AppWindow: Sized {
    fn on_create(window: &Arc<Window>) -> Result<Self>;
    fn on_redraw(&mut self, canvas: &mut Canvas, size: (u32, u32), delta: Duration) -> Result<()>;
    fn on_window_event(&mut self, event: crate::WindowEvent);
}
