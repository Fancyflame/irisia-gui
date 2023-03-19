use std::sync::mpsc;

use crate::runtime::{
    global::WindowRegiterMutex,
    rt_event::{RuntimeEvent, WindowReg},
};

use anyhow::Result;
use winit::window::WindowBuilder;

use super::Window;

impl Window {
    pub fn create<F>(f: F) -> Result<Self>
    where
        F: FnOnce(WindowBuilder) -> WindowBuilder + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        WindowRegiterMutex::lock().send(WindowReg::WindowCreate {
            build: Box::new(f),
            sender: tx,
        });

        match rx.recv()? {
            RuntimeEvent::WindowCreated { win } => Ok(Window {
                winit_window: win?,
                event_receiver: rx,
            }),
            _ => unreachable!("unexpected sys event"),
        }
    }
}
