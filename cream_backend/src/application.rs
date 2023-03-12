use std::time::Duration;

use anyhow::Result;
use skia_safe::Canvas;
use winit::event::WindowEvent;

pub trait Application {
    fn on_redraw(&mut self, canvas: &mut Canvas, size: (u32, u32), delta: Duration) -> Result<()>;
    fn on_window_event(&mut self, event: WindowEvent<'static>);
}
