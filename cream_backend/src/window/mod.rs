use std::{
    ops::{Deref, DerefMut},
    sync::mpsc,
};

use crate::{
    runtime::{
        global::WindowRegiterMutex,
        rt_event::{RuntimeEvent, WindowReg},
    },
    WinitWindow,
};

pub use winit::window::WindowBuilder;

pub mod create;
mod renderer;
pub mod run;

pub struct Window {
    winit_window: WinitWindow,
    event_receiver: mpsc::Receiver<RuntimeEvent>,
}

impl Deref for Window {
    type Target = WinitWindow;
    fn deref(&self) -> &Self::Target {
        &self.winit_window
    }
}

impl DerefMut for Window {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.winit_window
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        WindowRegiterMutex::lock().send(WindowReg::WindowDestroyed(self.winit_window.id()));
    }
}
