use std::{sync::mpsc::Sender, time::Instant};

use winit::{
    error::OsError,
    event::Event,
    window::{WindowBuilder, WindowId},
};

use crate::WinitWindow;

#[derive(Debug)]
pub(crate) enum RuntimeEvent {
    SysEvent(Event<'static, ()>),
    WindowCreated { win: Result<WinitWindow, OsError> },
}

pub(crate) enum WindowReg {
    WindowCreate {
        build: Box<dyn FnOnce(WindowBuilder) -> WindowBuilder + Send>,
        sender: Sender<RuntimeEvent>,
    },
    WindowDestroyed(WindowId),
    SetWait(Instant),
}
