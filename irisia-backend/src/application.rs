use std::time::Duration;

use anyhow::Result;
use skia_safe::Canvas;
use winit::event::WindowEvent;

pub trait AppWindow: 'static {
    fn on_redraw(&mut self, canvas: &Canvas, delta: Duration) -> Result<()>;
    fn on_window_event(&mut self, event: WindowEvent);
    fn on_destroy(&mut self);
}
