use std::{
    sync::{atomic::AtomicUsize, Arc},
    time::{Duration, Instant},
};

use anyhow::Result;
use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyboardInput},
};

use crate::{
    runtime::{rt_event::RuntimeEvent, TOKIO_RT},
    AppWindow, WindowEvent,
};

use super::{renderer::SurfaceProvider, Window};

impl Window {
    pub fn run<A>(self) -> Result<()>
    where
        A: AppWindow,
    {
        let window = Arc::new(self);

        let mut app = A::on_create(&window)?;

        let mut renderer = SurfaceProvider::new(window.clone())?;
        let mut last_frame_instant: Option<Instant> = None;

        let counter = Arc::new(AtomicUsize::new(0));

        {
            let counter = counter.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(1));
                let value = counter.swap(0, std::sync::atomic::Ordering::Relaxed);
                println!("{}:{}: {value}", file!(), line!());
            });
        }

        loop {
            let result = window.event_receiver.recv()?;
            counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if let RuntimeEvent::SysEvent(Event::WindowEvent { event, .. }) = &result {
                println!("recv");
            }

            match result {
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

                        renderer.render(window.inner_size(), |canvas, size| {
                            app.on_redraw(canvas, size, delta)
                        })?;
                        window.winit_window.request_redraw();
                    }

                    Event::WindowEvent {
                        window_id: _,
                        event,
                    } => {
                        app.on_window_event(event);
                    }

                    /*Event::NewEvents(StartCause::ResumeTimeReached {
                        requested_resume, ..
                    }) => {
                        window.timer.borrow_mut().update(requested_resume);
                    }*/
                    _ => {}
                },
                RuntimeEvent::WindowCreated { .. } => unreachable!("unexpect window created event"),
            }
        }
    }
}
