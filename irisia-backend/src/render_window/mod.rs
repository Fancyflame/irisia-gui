use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use anyhow::Result;
use tokio::sync::Mutex;
use winit::event::Event;

use crate::{AppWindow, WinitWindow};

use self::renderer::Renderer;

mod renderer;

enum RendererGetter {
    Pending(Pin<Box<dyn Future<Output = Result<Renderer>>>>),
    Error,
    Done(Renderer),
}

pub struct RenderWindow {
    app: Arc<Mutex<dyn AppWindow>>,
    window: Arc<WinitWindow>,
    renderer: RendererGetter,
    last_frame_instant: Option<Instant>,
}

impl RenderWindow {
    pub fn new(app: Arc<Mutex<dyn AppWindow>>, window: Arc<WinitWindow>) -> Result<Self> {
        let window_cloned = window.clone();
        let mut renderer_creation =
            RendererGetter::Pending(Box::pin(async move { Renderer::new(&window_cloned).await }));

        // start poll
        Self::renderer(&mut renderer_creation);

        Ok(RenderWindow {
            app: app as _,
            renderer: renderer_creation,
            window,
            last_frame_instant: None,
        })
    }

    fn renderer(renderer_getter: &mut RendererGetter) -> Option<&mut Renderer> {
        if let RendererGetter::Pending(future) = renderer_getter {
            let mut cx = Context::from_waker(futures::task::noop_waker_ref());
            match future.as_mut().poll(&mut cx) {
                Poll::Ready(Ok(r)) => {
                    *renderer_getter = RendererGetter::Done(r);
                }
                Poll::Ready(Err(e)) => {
                    eprintln!("renderer creation error: {e}");
                    *renderer_getter = RendererGetter::Error;
                    return None;
                }
                Poll::Pending => return None,
            }
        }

        match renderer_getter {
            RendererGetter::Done(r) => Some(r),
            RendererGetter::Error => None,
            RendererGetter::Pending(_) => unreachable!(),
        }
    }

    pub fn redraw(&mut self) {
        let delta = {
            let now = Instant::now();
            match self.last_frame_instant.replace(now) {
                Some(last) => now.duration_since(last),
                None => Duration::MAX,
            }
        };

        let mut app = match self.app.try_lock() {
            Ok(app) => app,
            Err(_) => self.app.blocking_lock(),
        };

        if let Some(renderer) = Self::renderer(&mut self.renderer) {
            if let Err(err) = renderer.resize(self.window.inner_size()) {
                eprintln!("cannot resize window: {err}");
            }

            if let Err(err) = renderer.render(|canvas, size| app.on_redraw(canvas, size, delta)) {
                eprintln!("render error: {err}");
            }
        }
    }

    pub fn handle_event(&mut self, event: Event<()>) {
        match event {
            Event::MainEventsCleared => {
                // restrict maximum drawing rate to 120fps
                const MIN_INTERVAL: Duration = Duration::from_millis(1000 / 120);

                if let Some(last) = self.last_frame_instant {
                    if last.elapsed() < MIN_INTERVAL {
                        return;
                    }
                }
            }

            Event::RedrawRequested(_) => {
                self.redraw();
            }

            Event::WindowEvent { event, .. } => {
                if let Some(static_event) = event.to_static() {
                    self.app.blocking_lock().on_window_event(static_event);
                }
            }

            _ => {}
        }
    }
}
