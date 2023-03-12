use std::{
    rc::Rc,
    time::{Duration, Instant},
};

use anyhow::Result;
use winit::{
    dpi::PhysicalSize,
    event::{Event, StartCause, WindowEvent},
};

use crate::{runtime::rt_event::RuntimeEvent, Application};

use super::{renderer::SurfaceProvider, Window};

impl Window {
    pub fn run_on<F, A>(self, f: F) -> Result<()>
    where
        F: FnOnce(&Rc<Window>) -> A,
        A: Application,
    {
        let window = Rc::new(self);

        let mut app = f(&window);

        let mut renderer = SurfaceProvider::new(&window.winit_window)?;

        let mut last_frame_instant: Option<Instant> = None;

        loop {
            match window.event_receiver.recv()? {
                RuntimeEvent::WindowCreated { .. } => unreachable!("unexpect window created event"),
                RuntimeEvent::SysEvent(event) => match event {
                    Event::LoopDestroyed => break Ok(()),

                    Event::RedrawRequested(_) => {
                        let delta = {
                            let now = Instant::now();
                            match last_frame_instant.replace(now.clone()) {
                                Some(ins) => now.duration_since(ins),
                                None => Duration::ZERO,
                            }
                        };

                        renderer.render(|canvas, size| app.on_redraw(canvas, size, delta))?;
                        window.winit_window.request_redraw();
                    }

                    Event::WindowEvent {
                        window_id: _,
                        event,
                    } => {
                        if let WindowEvent::Resized(PhysicalSize { width, height }) = &event {
                            renderer.resize(*width, *height);
                        }
                        app.on_window_event(event)
                    }

                    Event::NewEvents(StartCause::ResumeTimeReached {
                        requested_resume, ..
                    }) => {
                        window.timer.borrow_mut().update(requested_resume);
                    }

                    _ => {}
                },
            }
        }
    }
}
