use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use tokio::sync::Mutex;
use winit::event::{Event, WindowEvent};

use crate::{AppWindow, WinitWindow};

use self::renderer::Renderer;

mod renderer;

pub struct RenderWindow {
    app: Arc<Mutex<dyn AppWindow>>,
    window: Arc<WinitWindow>,
    renderer: Renderer,
    last_frame_instant: Option<Instant>,
}

impl RenderWindow {
    pub fn new(app: Arc<Mutex<dyn AppWindow>>, window: Arc<WinitWindow>) -> Result<Self> {
        Ok(RenderWindow {
            app: app as _,
            renderer: Renderer::new(&window)?,
            window,
            last_frame_instant: None,
        })
    }

    pub fn handle_event(&mut self, event: Event<()>) {
        match event {
            Event::MainEventsCleared => {
                self.window.request_redraw();
            }

            Event::RedrawRequested(_) => {
                let delta = {
                    let now = Instant::now();
                    match self.last_frame_instant.replace(now) {
                        Some(ins) => now.duration_since(ins),
                        None => Duration::ZERO,
                    }
                };

                let mut app = match self.app.try_lock() {
                    Ok(app) => app,
                    Err(_) => self.app.blocking_lock(),
                };

                if let Err(err) = self
                    .renderer
                    .render(|canvas, size| app.on_redraw(canvas, size, delta))
                {
                    eprintln!("render error: {err}");
                };
            }

            Event::WindowEvent { event, .. } => {
                if let WindowEvent::Resized(_) = &event {
                    if let Err(err) = self.renderer.resize(self.window.inner_size()) {
                        eprintln!("cannot resize window: {err}");
                    }
                }

                if let Some(static_event) = event.to_static() {
                    self.app.blocking_lock().on_window_event(static_event);
                }
            }

            _ => {}
        }
    }
}
