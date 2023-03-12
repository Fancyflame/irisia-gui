use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    sync::mpsc,
    time::{Duration, Instant},
};

use crate::{
    runtime::{
        global::WindowRegiterMutex,
        rt_event::{RuntimeEvent, WindowReg},
    },
    WinitWindow,
};

use self::timer::{CancelHandle, Timer};

pub mod create;
mod renderer;
pub mod run;
mod timer;

pub struct Window {
    winit_window: WinitWindow,
    event_receiver: mpsc::Receiver<RuntimeEvent>,
    timer: RefCell<Timer>,
}

impl Window {
    pub fn set_timeout<F>(&self, f: F, timeout: Duration) -> CancelHandle
    where
        F: FnMut() + 'static,
    {
        self.timer.borrow_mut().set_timeout(f, timeout)
    }

    pub fn set_timeout_at<F>(&self, f: F, instant: Instant) -> CancelHandle
    where
        F: FnOnce() + 'static,
    {
        self.timer.borrow_mut().set_timeout_at(f, instant)
    }

    pub fn set_interval<F>(&self, f: F, interval: Duration) -> CancelHandle
    where
        F: FnMut() + 'static,
    {
        self.timer.borrow_mut().set_interval(f, interval)
    }

    pub fn cancel_timeout(&self, handle: CancelHandle) -> bool {
        self.timer.borrow_mut().cancel(handle)
    }
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
