use std::time::Duration;

use anyhow::Result;
use skia_safe::Canvas;
use winit::{dpi::PhysicalSize, event::WindowEvent};

pub trait AppWindow: 'static {
    fn on_redraw(
        &mut self,
        canvas: &Canvas,
        delta: Duration,
        window_inner_size: PhysicalSize<u32>,
    ) -> Result<()>;
    fn on_window_event(&mut self, event: WindowEvent, window_inner_size: PhysicalSize<u32>);
    fn on_destroy(&mut self);
}
