use std::time::Duration;

use anyhow::Result;
use skia_safe::Canvas;

pub trait AppWindow: 'static {
    fn on_redraw(&mut self, canvas: &mut Canvas, delta: Duration) -> Result<()>;
    fn on_window_event(&mut self, event: crate::StaticWindowEvent);
    fn on_destroy(&mut self);
}
