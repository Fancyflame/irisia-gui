use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use pixels::Pixels;
use winit::event::WindowEvent;

use crate::{runtime::rt_event::AppBuildFn, AppWindow, WinitWindow};

use super::renderer::Renderer;

pub struct RenderWindow {
    app: Box<dyn AppWindow>,
    window: Arc<WinitWindow>,
    renderer: Renderer,
    last_frame_instant: Option<Instant>,
}

impl RenderWindow {
    pub fn new(app: AppBuildFn, pixels: Pixels, window: Arc<WinitWindow>) -> Result<Self> {
        Ok(RenderWindow {
            app: app(),
            renderer: Renderer::new(pixels, &window)?,
            window,
            last_frame_instant: None,
        })
    }

    pub fn redraw(&mut self) {
        let delta = {
            let now = Instant::now();
            match self.last_frame_instant.replace(now) {
                Some(last) => now.duration_since(last),
                None => Duration::MAX,
            }
        };

        if let Err(err) = self.renderer.resize(self.window.inner_size()) {
            eprintln!("cannot resize window: {err}");
        }

        if let Err(err) = self
            .renderer
            .render(|canvas| self.app.on_redraw(canvas, delta, self.window.inner_size()))
        {
            eprintln!("render error: {err}");
        }
    }

    pub fn handle_event(&mut self, event: WindowEvent) {
        self.app.on_window_event(event, self.window.inner_size());
    }
}

impl Drop for RenderWindow {
    fn drop(&mut self) {
        self.app.on_destroy();
    }
}
