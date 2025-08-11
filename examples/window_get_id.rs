use std::{
    sync::{atomic::AtomicU32, Arc},
    time::Duration,
};

use irisia::{
    anyhow::Result,
    winit::{
        application::ApplicationHandler,
        event::{Event, StartCause, WindowEvent},
        event_loop::EventLoop,
        window::Window,
    },
    WinitWindow,
};

struct App {
    window: Option<Window>,
    counter: Arc<AtomicU32>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &irisia::winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &irisia::winit::event_loop::ActiveEventLoop,
        window_id: irisia::winit::window::WindowId,
        event: irisia::winit::event::WindowEvent,
    ) {
        if let WindowEvent::RedrawRequested = event {
            self.counter
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.window.as_mut().unwrap().request_redraw();
        }
    }

    fn new_events(
        &mut self,
        event_loop: &irisia::winit::event_loop::ActiveEventLoop,
        cause: irisia::winit::event::StartCause,
    ) {
        if let StartCause::Init = cause {
            let window = event_loop
                .create_window(Window::default_attributes())
                .unwrap();

            {
                let counter = self.counter.clone();
                std::thread::spawn(move || loop {
                    std::thread::sleep(Duration::from_secs(1));
                    let fps = counter.swap(0, std::sync::atomic::Ordering::Relaxed);
                    println!("{fps}fps");
                })
            };

            self.window.insert(window).request_redraw();
        }
    }
}

fn main() -> Result<()> {
    EventLoop::new()?
        .run_app(&mut App {
            window: None,
            counter: Arc::new(AtomicU32::new(0)),
        })
        .unwrap();
    Ok(())
}
